use crate::db::Db;
use crate::models::Customer;
use tauri::State;

fn row_to_customer(r: &rusqlite::Row) -> rusqlite::Result<Customer> {
    Ok(Customer {
        id: r.get(0)?,
        name: r.get(1)?,
        phone: r.get(2)?,
        address: r.get(3)?,
        email: r.get(4)?,
    })
}

#[tauri::command]
pub fn customers_list(db: State<Db>, search: Option<String>) -> Result<Vec<Customer>, String> {
    let conn = db.0.lock().unwrap();
    let q = search.unwrap_or_default().trim().to_lowercase();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, phone, address, email FROM customers
             WHERE ?1 = '' OR lower(name) LIKE '%'||?1||'%' OR lower(coalesce(phone,'')) LIKE '%'||?1||'%'
             ORDER BY name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([q], row_to_customer).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn customers_create(db: State<Db>, customer: Customer) -> Result<i64, String> {
    let conn = db.0.lock().unwrap();
    if customer.name.trim().is_empty() {
        return Err("Le nom du client est requis".into());
    }
    conn.execute(
        "INSERT INTO customers (name, phone, address, email) VALUES (?1,?2,?3,?4)",
        rusqlite::params![customer.name.trim(), customer.phone, customer.address, customer.email],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn customers_update(db: State<Db>, id: i64, customer: Customer) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE customers SET name=?1, phone=?2, address=?3, email=?4 WHERE id=?5",
        rusqlite::params![customer.name.trim(), customer.phone, customer.address, customer.email, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
