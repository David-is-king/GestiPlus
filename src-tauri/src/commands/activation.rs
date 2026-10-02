use crate::db::app_data_dir;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const ACTIVATION_KEY: &str = "GESTIPLUS-2026-X7K9-P4M2";

#[derive(Debug, Serialize, Deserialize)]
struct ActivationRecord {
    activated: bool,
    machine_id: String,
    activated_at: String,
}

fn activation_path() -> PathBuf {
    let mut path = app_data_dir();
    path.push("activation.dat");
    path
}

fn machine_id() -> String {
    let computer = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown-computer".into());
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "unknown-user".into());
    format!("{}::{}", computer, user)
}

#[tauri::command]
pub fn activation_status() -> bool {
    let path = activation_path();
    let Ok(contents) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(record) = serde_json::from_str::<ActivationRecord>(&contents) else {
        return false;
    };
    record.activated && record.machine_id == machine_id()
}

#[tauri::command]
pub fn activation_activate(key: String) -> Result<(), String> {
    let normalized = key.trim().to_ascii_uppercase();
    if normalized != ACTIVATION_KEY {
        return Err("Clé d'activation incorrecte".into());
    }

    let record = ActivationRecord {
        activated: true,
        machine_id: machine_id(),
        activated_at: Local::now().to_rfc3339(),
    };
    let contents = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    std::fs::write(activation_path(), contents)
        .map_err(|e| format!("Impossible d'enregistrer l'activation : {}", e))?;
    Ok(())
}
