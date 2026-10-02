use crate::db::Db;
use crate::models::StoreSettings;
use tauri::State;

#[tauri::command]
pub fn settings_get(db: State<Db>) -> Result<StoreSettings, String> {
    let conn = db.0.lock().unwrap();
    conn.query_row(
        "SELECT name, emplacement, quartier, rue, phone1, phone2, phone3, email, currency, slogan, commercial_name, logo_path
         FROM store_settings WHERE id = 1",
        [],
        |r| {
            Ok(StoreSettings {
                name: r.get(0)?,
                emplacement: r.get(1)?,
                quartier: r.get(2)?,
                rue: r.get(3)?,
                phone1: r.get(4)?,
                phone2: r.get(5)?,
                phone3: r.get(6)?,
                email: r.get(7)?,
                currency: r.get(8)?,
                slogan: r.get(9)?,
                commercial_name: r.get(10)?,
                logo_path: r.get(11)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn settings_update(db: State<Db>, settings: StoreSettings) -> Result<(), String> {
    if settings.name.trim().is_empty() {
        return Err("Le nom de la boutique est requis".into());
    }
    let conn = db.0.lock().unwrap();
    conn.execute(
        "UPDATE store_settings SET name=?1, emplacement=?2, quartier=?3, rue=?4, phone1=?5,
            phone2=?6, phone3=?7, email=?8, currency=?9, slogan=?10, commercial_name=?11, logo_path=?12 WHERE id = 1",
        rusqlite::params![
            settings.name.trim(),
            settings.emplacement,
            settings.quartier,
            settings.rue,
            settings.phone1,
            settings.phone2,
            settings.phone3,
            settings.email,
            settings.currency,
            settings.slogan,
            settings.commercial_name,
            settings.logo_path,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
