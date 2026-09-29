use crate::db::Db;
use crate::models::Supplier;
use tauri::State;

fn row_to_supplier(r: &rusqlite::Row) -> rusqlite::Result<Supplier> {
    Ok(Supplier {
        id: r.get(0)?,
        name: r.get(1)?,
        phone1: r.get(2)?,
        phone2: r.get(3)?,
        address: r.get(4)?,
        email: r.get(5)?,
        notes: r.get(6)?,
    })
}

#[tauri::command]
pub fn suppliers_list(db: State<Db>, search: Option<String>) -> Result<Vec<Supplier>, String> {
    let conn = db.0.lock().unwrap();
    let q = search.unwrap_or_default().trim().to_lowercase();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, phone1, phone2, address, email, notes FROM suppliers
             WHERE ?1 = '' OR lower(name) LIKE '%'||?1||'%' OR lower(coalesce(phone1,'')) LIKE '%'||?1||'%'
             ORDER BY name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([q], row_to_supplier).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn suppliers_create(db: State<Db>, supplier: Supplier) -> Result<i64, String> {
    let conn = db.0.lock().unwrap();
    if supplier.name.trim().is_empty() {
        return Err("Le nom du fournisseur est requis".into());
    }
    conn.execute(
        "INSERT INTO suppliers (name, phone1, phone2, address, email, notes) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![supplier.name.trim(), supplier.phone1, supplier.phone2, supplier.address, supplier.email, supplier.notes],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn suppliers_update(db: State<Db>, id: i64, supplier: Supplier) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE suppliers SET name=?1, phone1=?2, phone2=?3, address=?4, email=?5, notes=?6 WHERE id=?7",
        rusqlite::params![supplier.name.trim(), supplier.phone1, supplier.phone2, supplier.address, supplier.email, supplier.notes, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
