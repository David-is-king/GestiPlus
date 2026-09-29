use crate::db::{normalize, Db};
use crate::models::Category;
use tauri::State;

#[tauri::command]
pub fn categories_list(db: State<Db>) -> Result<Vec<Category>, String> {
    let conn = db.0.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, created_at FROM categories ORDER BY name COLLATE NOCASE")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                name: r.get(1)?,
                created_at: r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn categories_create(db: State<Db>, name: String) -> Result<i64, String> {
    let conn = db.0.lock().unwrap();
    let norm = normalize(&name);
    if norm.is_empty() {
        return Err("Le nom de la catégorie est requis".into());
    }
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM categories WHERE name_normalized = ?1",
            [&norm],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        return Err("Cette catégorie existe déjà".into());
    }
    conn.execute(
        "INSERT INTO categories (name, name_normalized) VALUES (?1, ?2)",
        rusqlite::params![name.trim(), norm],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn categories_update(db: State<Db>, id: i64, name: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let norm = normalize(&name);

    if norm.is_empty() {
        return Err("Le nom de la catégorie ne peut pas être vide".into());
    }
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM categories WHERE name_normalized = ?1 AND id != ?2",
            rusqlite::params![norm, id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        return Err("Cette catégorie existe déjà".into());
    }
    conn.execute(
        "UPDATE categories SET name = ?1, name_normalized = ?2 WHERE id = ?3",
        rusqlite::params![name.trim(), norm, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Une catégorie utilisée par des produits ne peut pas être supprimée (règle du cahier des charges 6).
#[tauri::command]
pub fn categories_delete(db: State<Db>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let used: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE category_id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if used > 0 {
        return Err("Impossible de supprimer : des produits utilisent cette catégorie".into());
    }
    conn.execute("DELETE FROM categories WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
