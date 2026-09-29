use crate::db::Db;
use crate::models::StockMovement;
use tauri::State;

#[tauri::command]
pub fn movements_list(
    db: State<Db>,
    product_id: Option<i64>,
    movement_type: Option<String>,
) -> Result<Vec<StockMovement>, String> {
    let conn = db.0.lock().unwrap();
    let sql = "SELECT m.id, m.product_id, p.name, m.type, m.quantity, m.stock_before, m.stock_after,
        m.motif, m.reference, m.created_at
        FROM stock_movements m
        LEFT JOIN products p ON p.id = m.product_id
        WHERE (?1 IS NULL OR m.product_id = ?1)
          AND (?2 IS NULL OR m.type = ?2)
        ORDER BY m.created_at DESC, m.id DESC
        LIMIT 500";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![product_id, movement_type], |r| {
            Ok(StockMovement {
                id: r.get(0)?,
                product_id: r.get(1)?,
                product_name: r.get(2)?,
                movement_type: r.get(3)?,
                quantity: r.get(4)?,
                stock_before: r.get(5)?,
                stock_after: r.get(6)?,
                motif: r.get(7)?,
                reference: r.get(8)?,
                created_at: r.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}
