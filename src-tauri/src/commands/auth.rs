use crate::db::Db;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[tauri::command]
pub fn auth_login(db: State<Db>, username: String, password: String) -> Result<UserInfo, String> {
    let conn = db.0.lock().unwrap();
    let result: Result<(i64, String, String), _> = conn.query_row(
        "SELECT id, username, password_hash FROM users WHERE username = ?1 COLLATE NOCASE",
        [&username],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    );

    match result {
        Ok((id, uname, hash)) => {
            if bcrypt::verify(&password, &hash).unwrap_or(false) {
                Ok(UserInfo { id, username: uname })
            } else {
                Err("Identifiants incorrects".into())
            }
        }
        Err(_) => Err("Identifiants incorrects".into()),
    }
}

#[tauri::command]
pub fn auth_change_password(
    db: State<Db>,
    user_id: i64,
    old_password: String,
    new_password: String,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let hash: String = conn
        .query_row("SELECT password_hash FROM users WHERE id = ?1", [user_id], |r| r.get(0))
        .map_err(|_| "Utilisateur introuvable".to_string())?;

    if !bcrypt::verify(&old_password, &hash).unwrap_or(false) {
        return Err("Ancien mot de passe incorrect".into());
    }
    if new_password.trim().len() < 4 {
        return Err("Le nouveau mot de passe est trop court".into());
    }
    let new_hash = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE users SET password_hash = ?1 WHERE id = ?2",
        rusqlite::params![new_hash, user_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn auth_change_username(db: State<Db>, user_id: i64, new_username: String) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE username = ?1 COLLATE NOCASE AND id != ?2",
            rusqlite::params![new_username, user_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if exists > 0 {
        return Err("Ce nom d'utilisateur est déjà pris".into());
    }
    conn.execute(
        "UPDATE users SET username = ?1 WHERE id = ?2",
        rusqlite::params![new_username, user_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
