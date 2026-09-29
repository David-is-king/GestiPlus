use crate::db::{backups_dir, db_path, Db};
use tauri::State;

#[tauri::command]
pub fn backup_create(db: State<Db>) -> Result<String, String> {
    let conn = db.0.lock().unwrap();
    // S'assure que toutes les écritures WAL sont bien appliquées avant la copie.
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").ok();

    let date = chrono::Local::now().format("%Y-%m-%d_%Hh%M").to_string();
    let mut dest = backups_dir();
    dest.push(format!("backup_boutique_{}.db", date));

    std::fs::copy(db_path(), &dest).map_err(|e| format!("Échec de la sauvegarde : {}", e))?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn backup_list() -> Result<Vec<String>, String> {
    let dir = backups_dir();
    let mut files: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "db").unwrap_or(false))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    files.sort();
    files.reverse();
    Ok(files)
}

/// Restaure la base à partir d'un fichier de sauvegarde. L'application doit être redémarrée
/// après cette opération pour que la nouvelle base soit prise en compte proprement.
#[tauri::command]
pub fn backup_restore(db: State<Db>, backup_path: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    drop(conn); // libère le verrou avant de manipuler le fichier
    std::fs::copy(&backup_path, db_path()).map_err(|e| format!("Échec de la restauration : {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn backup_get_db_path() -> String {
    db_path().to_string_lossy().to_string()
}
