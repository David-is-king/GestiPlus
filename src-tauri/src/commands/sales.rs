use crate::db::Db;
use crate::models::{Sale, SaleItem, SaleItemInput};
use chrono::Datelike;
use tauri::State;

fn load_items(conn: &rusqlite::Connection, sale_id: i64) -> Vec<SaleItem> {
    let mut stmt = conn
        .prepare("SELECT id, product_id, product_name, unit_price, quantity, subtotal FROM sale_items WHERE sale_id = ?1")
        .unwrap();
    stmt.query_map([sale_id], |r| {
        Ok(SaleItem {
            id: r.get(0)?,
            product_id: r.get(1)?,
            product_name: r.get(2)?,
            unit_price: r.get(3)?,
            quantity: r.get(4)?,
            subtotal: r.get(5)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

fn next_sale_number(tx: &rusqlite::Transaction, year: i32) -> Result<String, String> {
    tx.execute(
        "INSERT INTO invoice_counters (year, last_number) VALUES (?1, 0)
         ON CONFLICT(year) DO NOTHING",
        [year],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE invoice_counters SET last_number = last_number + 1 WHERE year = ?1",
        [year],
    )
    .map_err(|e| e.to_string())?;
    let n: i64 = tx
        .query_row("SELECT last_number FROM invoice_counters WHERE year = ?1", [year], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(format!("V-{}-{:06}", year, n))
}

/// Crée une vente : vérifie le stock disponible, décrémente le stock, crée les mouvements de
/// sortie et calcule le total. Toute l'opération est transactionnelle.
#[tauri::command]
pub fn sales_create(
    db: State<Db>,
    customer_id: Option<i64>,
    items: Vec<SaleItemInput>,
) -> Result<Sale, String> {
    if items.is_empty() {
        return Err("Une vente doit comporter au moins un produit".into());
    }
    let mut conn = db.0.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut total = 0.0f64;
    let mut prepared: Vec<(i64, String, f64, f64, i64, i64)> = Vec::new();

    for item in &items {
        if item.quantity <= 0 {
            return Err("La quantité doit être positive".into());
        }
        let (name, sale_price, purchase_price, stock): (String, f64, f64, i64) = tx
            .query_row(
                "SELECT name, sale_price, purchase_price, stock_qty FROM products WHERE id = ?1",
                [item.product_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(|_| format!("Produit introuvable (id {})", item.product_id))?;

        if item.quantity > stock {
            return Err(format!("Stock insuffisant pour {} (disponible : {})", name, stock));
        }
        total += sale_price * item.quantity as f64;
        prepared.push((item.product_id, name, sale_price, purchase_price, stock, item.quantity));
    }

    let year = chrono::Local::now().year();
    let sale_number = next_sale_number(&tx, year)?;

    // Récupération facultative du nom du client s'il est sélectionné dans la BDD
    let customer_name: Option<String> = customer_id.and_then(|cid| {
        tx.query_row("SELECT name FROM customers WHERE id = ?1", [cid], |r| r.get(0)).ok()
    });

    tx.execute(
        "INSERT INTO sales (sale_number, customer_id, total, status) VALUES (?1, ?2, ?3, 'validee')",
        rusqlite::params![sale_number, customer_id, total],
    )
    .map_err(|e| e.to_string())?;
    let sale_id = tx.last_insert_rowid();

    for (product_id, name, sale_price, purchase_price, stock_before, qty) in &prepared {
        let subtotal = sale_price * *qty as f64;
        tx.execute(
            "INSERT INTO sale_items (sale_id, product_id, product_name, unit_price, purchase_price_at_sale, quantity, subtotal)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![sale_id, product_id, name, sale_price, purchase_price, qty, subtotal],
        )
        .map_err(|e| e.to_string())?;

        let stock_after = stock_before - qty;
        tx.execute(
            "UPDATE products SET stock_qty = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![stock_after, product_id],
        )
        .map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT INTO stock_movements (product_id, type, quantity, stock_before, stock_after, motif, reference)
             VALUES (?1, 'sortie', ?2, ?3, ?4, 'Vente', ?5)",
            rusqlite::params![product_id, qty, stock_before, stock_after, sale_number],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    let items_out = load_items(&conn, sale_id);

    Ok(Sale {
        id: sale_id,
        sale_number,
        customer_id,
        customer_name,
        total,
        status: "validee".into(),
        created_at: chrono::Local::now().to_rfc3339(),
        items: items_out,
    })
}

#[tauri::command]
pub fn sales_list(
    db: State<Db>,
    date_from: Option<String>,
    date_to: Option<String>,
    search: Option<String>,
) -> Result<Vec<Sale>, String> {
    let conn = db.0.lock().unwrap();
    let q = search.unwrap_or_default().trim().to_lowercase();
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.sale_number, s.customer_id, c.name, s.total, s.status, s.created_at
             FROM sales s LEFT JOIN customers c ON c.id = s.customer_id
             WHERE (?1 IS NULL OR date(s.created_at) >= date(?1))
               AND (?2 IS NULL OR date(s.created_at) <= date(?2))
               AND (?3 = '' OR lower(s.sale_number) LIKE '%'||?3||'%' OR lower(coalesce(c.name,'')) LIKE '%'||?3||'%')
             ORDER BY s.created_at DESC LIMIT 500",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![date_from, date_to, q], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, f64>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut sales = Vec::new();
    for row in rows.filter_map(|r| r.ok()) {
        let (id, sale_number, customer_id, customer_name, total, status, created_at) = row;
        sales.push(Sale {
            id,
            sale_number,
            customer_id,
            customer_name,
            total,
            status,
            created_at,
            items: load_items(&conn, id),
        });
    }
    Ok(sales)
}

/// Annule une vente : restaure le stock, enregistre un mouvement inverse, passe le statut à
/// 'annulee'. La vente n'est jamais supprimée (conservation de l'historique).
#[tauri::command]
pub fn sales_cancel(db: State<Db>, sale_id: i64) -> Result<(), String> {
    let mut conn = db.0.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let status: String = tx
        .query_row("SELECT status FROM sales WHERE id = ?1", [sale_id], |r| r.get(0))
        .map_err(|_| "Vente introuvable".to_string())?;
    if status == "annulee" {
        return Err("Cette vente est déjà annulée".into());
    }

    let mut stmt = tx
        .prepare("SELECT product_id, quantity FROM sale_items WHERE sale_id = ?1")
        .map_err(|e| e.to_string())?;
    let items: Vec<(i64, i64)> = stmt
        .query_map([sale_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    for (product_id, qty) in items {
        let stock_before: i64 = tx
            .query_row("SELECT stock_qty FROM products WHERE id = ?1", [product_id], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let stock_after = stock_before + qty;
        tx.execute(
            "UPDATE products SET stock_qty = ?1 WHERE id = ?2",
            rusqlite::params![stock_after, product_id],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO stock_movements (product_id, type, quantity, stock_before, stock_after, motif)
             VALUES (?1, 'entree', ?2, ?3, ?4, 'Annulation de vente')",
            rusqlite::params![product_id, qty, stock_before, stock_after],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.execute(
        "UPDATE sales SET status = 'annulee', cancelled_at = datetime('now') WHERE id = ?1",
        [sale_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}