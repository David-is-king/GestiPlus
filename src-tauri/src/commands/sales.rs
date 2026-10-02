use crate::db::{normalize, Db};
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

/// Retrouve un client existant par (nom normalisé + téléphone), ou le crée s'il n'existe pas.
/// C'est ce qui empêche de dupliquer un client homonyme : deux "Mariam Traoré" avec des
/// téléphones différents donnent deux fiches distinctes ; le même nom + le même téléphone
/// réutilise toujours la même fiche.
fn find_or_create_customer(tx: &rusqlite::Transaction, name: &str, phone: &str) -> Result<i64, String> {
    let name = name.trim();
    let phone = phone.trim();

    if name.is_empty() {
        return Err("Le nom du client est requis pour ce mode de paiement".into());
    }
    if phone.is_empty() {
        return Err("Le numéro de téléphone est obligatoire pour accorder un crédit ou un paiement partiel".into());
    }

    let norm_name = normalize(name);

    // On récupère tous les clients partageant ce téléphone, puis on compare le nom normalisé :
    // ça couvre le cas où le même client est resaisi avec une casse ou des espaces différents.
    let mut stmt = tx
        .prepare("SELECT id, name FROM customers WHERE trim(coalesce(phone,'')) = ?1")
        .map_err(|e| e.to_string())?;
    let candidates: Vec<(i64, String)> = stmt
        .query_map([phone], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    for (id, existing_name) in &candidates {
        if normalize(existing_name) == norm_name {
            return Ok(*id);
        }
    }

    tx.execute(
        "INSERT INTO customers (name, phone) VALUES (?1, ?2)",
        rusqlite::params![name, phone],
    )
    .map_err(|e| e.to_string())?;
    Ok(tx.last_insert_rowid())
}

/// Crée une vente : vérifie le stock disponible, décrémente le stock, crée les mouvements de
/// sortie, calcule le total et gère le paiement (payée / partielle / à crédit).
///
/// Règle clé : si `payment_status` n'est pas "payee", il faut un client identifié par
/// téléphone — soit `customer_id` (déjà en base), soit `customer_name` + `customer_phone`
/// (recherché ou créé automatiquement). Une vente payée intégralement peut toujours rester
/// anonyme ou n'avoir qu'un nom libre non enregistré, comme avant.
#[tauri::command]
pub fn sales_create(
    db: State<Db>,
    customer_id: Option<i64>,
    customer_name: Option<String>,
    customer_phone: Option<String>,
    payment_status: String,
    amount_paid: Option<f64>,
    items: Vec<SaleItemInput>,
) -> Result<Sale, String> {
    if items.is_empty() {
        return Err("Une vente doit comporter au moins un produit".into());
    }
    if !["payee", "partielle", "credit"].contains(&payment_status.as_str()) {
        return Err(format!("Statut de paiement inconnu : {}", payment_status));
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

    // --- Résolution du client ---
    // Un customer_id explicite (sélectionné dans la liste) est toujours prioritaire.
    // Sinon, pour un crédit/paiement partiel, on retrouve ou crée le client via nom+téléphone.
    // Pour une vente payée intégralement sans customer_id, on reste sur le comportement
    // existant : le nom éventuel n'est qu'une étiquette pour la facture, rien n'est enregistré.
    let effective_customer_id: Option<i64> = if let Some(cid) = customer_id {
        Some(cid)
    } else if payment_status != "payee" {
        let name = customer_name.clone().unwrap_or_default();
        let phone = customer_phone.clone().unwrap_or_default();
        Some(find_or_create_customer(&tx, &name, &phone)?)
    } else {
        None
    };

    // --- Résolution du montant payé ---
    let final_amount_paid = match payment_status.as_str() {
        "payee" => total,
        "credit" => 0.0,
        "partielle" => {
            let ap = amount_paid.unwrap_or(0.0);
            if ap <= 0.0 {
                return Err("Le montant payé doit être supérieur à 0 pour un paiement partiel".into());
            }
            if ap >= total {
                return Err(
                    "Le montant payé couvre déjà tout le total : choisissez « Payé » plutôt que « Partiel »"
                        .into(),
                );
            }
            ap
        }
        _ => unreachable!(),
    };

    let year = chrono::Local::now().year();
    let sale_number = next_sale_number(&tx, year)?;

    tx.execute(
        "INSERT INTO sales (sale_number, customer_id, total, status, amount_paid, payment_status)
         VALUES (?1, ?2, ?3, 'validee', ?4, ?5)",
        rusqlite::params![sale_number, effective_customer_id, total, final_amount_paid, payment_status],
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

    // Nom à renvoyer pour affichage immédiat côté UI : celui du client en base s'il y en a un,
    // sinon le nom libre fourni (cas "payée + nom non enregistré").
    let customer_name_out: Option<String> = if let Some(cid) = effective_customer_id {
        tx.query_row("SELECT name FROM customers WHERE id = ?1", [cid], |r| r.get(0)).ok()
    } else {
        customer_name.clone()
    };

    tx.commit().map_err(|e| e.to_string())?;

    let items_out = load_items(&conn, sale_id);

    Ok(Sale {
        id: sale_id,
        sale_number,
        customer_id: effective_customer_id,
        customer_name: customer_name_out,
        total,
        status: "validee".into(),
        amount_paid: final_amount_paid,
        payment_status,
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
            "SELECT s.id, s.sale_number, s.customer_id, c.name, s.total, s.status, s.created_at,
                    s.amount_paid, s.payment_status
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
                r.get::<_, f64>(7)?,
                r.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);

    let mut sales = Vec::new();
    for (id, sale_number, customer_id, customer_name, total, status, created_at, stored_amount_paid, stored_payment_status) in rows {
        let payment_state = crate::commands::credits::sale_balance(&conn, id).unwrap_or(crate::commands::credits::SaleBalance {
            amount_paid: stored_amount_paid,
            balance_due: (total - stored_amount_paid).max(0.0),
        });
        let payment_status = if payment_state.balance_due <= 0.000_001 {
            "payee".to_string()
        } else if payment_state.amount_paid > 0.0 {
            "partielle".to_string()
        } else {
            stored_payment_status
        };
        sales.push(Sale {
            id,
            sale_number,
            customer_id,
            customer_name,
            total,
            status,
            amount_paid: payment_state.amount_paid,
            payment_status,
            created_at,
            items: load_items(&conn, id),
        });
    }
    Ok(sales)
}

/// Annule une vente : restaure le stock, enregistre un mouvement inverse, passe le statut à
/// 'annulee'. La vente n'est jamais supprimée (conservation de l'historique, y compris la
/// créance éventuelle qui disparaît logiquement puisque la vente est annulée).
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
