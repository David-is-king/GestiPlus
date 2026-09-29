use crate::db::{normalize, Db};
use crate::models::{Product, ProductInput};
use tauri::State;

const SELECT_BASE: &str = "SELECT p.id, p.name, p.category_id, c.name, p.purchase_price, p.sale_price,
    p.stock_qty, p.alert_threshold, p.supplier_id, s.name, p.supplier_ref, p.internal_ref, p.description
    FROM products p
    LEFT JOIN categories c ON c.id = p.category_id
    LEFT JOIN suppliers s ON s.id = p.supplier_id";

fn row_to_product(r: &rusqlite::Row) -> rusqlite::Result<Product> {
    Ok(Product {
        id: r.get(0)?,
        name: r.get(1)?,
        category_id: r.get(2)?,
        category_name: r.get(3)?,
        purchase_price: r.get(4)?,
        sale_price: r.get(5)?,
        stock_qty: r.get(6)?,
        alert_threshold: r.get(7)?,
        supplier_id: r.get(8)?,
        supplier_name: r.get(9)?,
        supplier_ref: r.get(10)?,
        internal_ref: r.get(11)?,
        description: r.get(12)?,
    })
}

#[tauri::command]
pub fn products_list(db: State<Db>, search: Option<String>) -> Result<Vec<Product>, String> {
    let conn = db.0.lock().unwrap();
    let query = search.unwrap_or_default();
    let norm = normalize(&query);

    let sql = format!("{} WHERE (?1 = '' OR lower(p.name) LIKE '%' || ?1 || '%' OR lower(coalesce(p.internal_ref,'')) LIKE '%' || ?1 || '%') ORDER BY p.name COLLATE NOCASE", SELECT_BASE);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([norm], row_to_product)
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn products_get(db: State<Db>, id: i64) -> Result<Product, String> {
    let conn = db.0.lock().unwrap();
    let sql = format!("{} WHERE p.id = ?1", SELECT_BASE);
    conn.query_row(&sql, [id], row_to_product)
        .map_err(|_| "Produit introuvable".to_string())
}

#[tauri::command]
pub fn products_create(db: State<Db>, input: ProductInput) -> Result<i64, String> {
    let conn = db.0.lock().unwrap();
    let norm = normalize(&input.name);
    if norm.is_empty() {
        return Err("Le nom du produit est requis".into());
    }
        // Validation : Prix de vente obligatoire
    if input.sale_price <= 0.0 {
        return Err("Le prix de vente est obligatoire et doit être supérieur à zéro".into());
    }

    // Validation : Quantité initiale obligatoire
    if input.stock_qty <= 0 {
        return Err("La quantité initiale est obligatoire et doit être supérieure à zéro".into());
    }
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE name_normalized = ?1",
            [&norm],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        return Err("Un produit avec ce nom existe déjà".into());
    }

    conn.execute(
        "INSERT INTO products (name, name_normalized, category_id, purchase_price, sale_price,
            stock_qty, alert_threshold, supplier_id, supplier_ref, internal_ref, description)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        rusqlite::params![
            input.name.trim(),
            norm,
            input.category_id,
            input.purchase_price,
            input.sale_price,
            input.stock_qty,
            input.alert_threshold,
            input.supplier_id,
            input.supplier_ref,
            input.internal_ref,
            input.description,
        ],
    )
    .map_err(|e| e.to_string())?;

    let product_id = conn.last_insert_rowid();

    if input.stock_qty > 0 {
        conn.execute(
            "INSERT INTO stock_movements (product_id, type, quantity, stock_before, stock_after, motif)
             VALUES (?1, 'entree', ?2, 0, ?2, 'Stock initial')",
            rusqlite::params![product_id, input.stock_qty],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(product_id)
}

#[tauri::command]
pub fn products_update(db: State<Db>, id: i64, input: ProductInput) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let norm = normalize(&input.name);

    // Validation : Empêcher de rendre le nom vide lors d'une modification
    if norm.is_empty() {
        return Err("Le nom du produit ne peut pas être vide".into());
    }

    // Validation : Prix de vente obligatoire lors d'une modification
    if input.sale_price <= 0.0 {
        return Err("Le prix de vente est obligatoire et doit être supérieur à zéro".into());
    }
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE name_normalized = ?1 AND id != ?2",
            rusqlite::params![norm, id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        return Err("Un produit avec ce nom existe déjà".into());
    }
    conn.execute(
        "UPDATE products SET name=?1, name_normalized=?2, category_id=?3, purchase_price=?4,
            sale_price=?5, alert_threshold=?6, supplier_id=?7, supplier_ref=?8, internal_ref=?9,
            description=?10, updated_at = datetime('now')
         WHERE id = ?11",
        rusqlite::params![
            input.name.trim(),
            norm,
            input.category_id,
            input.purchase_price,
            input.sale_price,
            input.alert_threshold,
            input.supplier_id,
            input.supplier_ref,
            input.internal_ref,
            input.description,
            id,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Ajoute ou retire une quantité au stock existant. Crée toujours un mouvement.
/// Transmettez une valeur positive pour ajouter, ou négative pour retirer (ex: -5).
#[tauri::command]
pub fn products_add_stock(db: State<Db>, id: i64, quantity: i64, motif: Option<String>) -> Result<i64, String> {
    if quantity == 0 {
        return Err("La quantité à ajouter ou retirer ne peut pas être égale à zéro".into());
    }

    let conn = db.0.lock().unwrap();
    let current: i64 = conn
        .query_row("SELECT stock_qty FROM products WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| "Produit introuvable".to_string())?;

    // Détermination du type de mouvement et vérification du stock en cas de retrait
    let (movement_type, default_motif) = if quantity > 0 {
        ("entree", "Réception")
    } else {
        let abs_qty = quantity.abs();
        if abs_qty > current {
            return Err(format!(
                "Stock insuffisant : vous souhaitez retirer {}, mais il n'y en a que {} en stock",
                abs_qty, current
            ));
        }
        ("sortie", "Retrait de stock")
    };

    let new_qty = current + quantity;

    conn.execute(
        "UPDATE products SET stock_qty = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![new_qty, id],
    )
    .map_err(|e| e.to_string())?;

    // On enregistre la valeur absolue dans la table 'stock_movements' pour garder la quantité positive dans l'historique
    conn.execute(
        "INSERT INTO stock_movements (product_id, type, quantity, stock_before, stock_after, motif)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            id,
            movement_type,
            quantity.abs(),
            current,
            new_qty,
            motif.unwrap_or_else(|| default_motif.into())
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(new_qty)
}

/// Corrige exceptionnellement le stock (inventaire). Crée un mouvement de type 'ajustement'.
#[tauri::command]
pub fn products_adjust_stock(db: State<Db>, id: i64, new_quantity: i64, motif: Option<String>) -> Result<(), String> {
    if new_quantity < 0 {
        return Err("La quantité ne peut pas être négative".into());
    }
    let conn = db.0.lock().unwrap();
    let current: i64 = conn
        .query_row("SELECT stock_qty FROM products WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| "Produit introuvable".to_string())?;
    conn.execute(
        "UPDATE products SET stock_qty = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![new_quantity, id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO stock_movements (product_id, type, quantity, stock_before, stock_after, motif)
         VALUES (?1, 'ajustement', ?2, ?3, ?4, ?5)",
        rusqlite::params![
            id,
            new_quantity - current,
            current,
            new_quantity,
            motif.unwrap_or_else(|| "Ajustement d'inventaire".into())
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn products_low_stock(db: State<Db>) -> Result<Vec<Product>, String> {
    let conn = db.0.lock().unwrap();
    let sql = format!("{} WHERE p.stock_qty <= p.alert_threshold ORDER BY p.stock_qty ASC", SELECT_BASE);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], row_to_product).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}
