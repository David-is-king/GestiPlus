#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod pdf;

use db::Db;
use std::sync::Mutex;

fn main() {
    tauri::Builder::default()
    
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Db(Mutex::new(db::init_connection())))
        .invoke_handler(tauri::generate_handler![
            commands::auth::auth_login,
            commands::auth::auth_change_password,
            commands::auth::auth_change_username,
            commands::categories::categories_list,
            commands::categories::categories_create,
            commands::categories::categories_update,
            commands::categories::categories_delete,
            commands::products::products_list,
            commands::products::products_get,
            commands::products::products_create,
            commands::products::products_update,
            commands::products::products_add_stock,
            commands::products::products_adjust_stock,
            commands::products::products_low_stock,
            commands::stock_movements::movements_list,
            commands::suppliers::suppliers_list,
            commands::suppliers::suppliers_create,
            commands::suppliers::suppliers_update,
            commands::customers::customers_list,
            commands::customers::customers_create,
            commands::customers::customers_update,
            commands::credits::credits_list,
            commands::credits::credits_get,
            commands::credits::credit_payment_create,
            commands::sales::sales_create,
            commands::sales::sales_list,
            commands::sales::sales_cancel,
            commands::invoices::invoices_open_pdf_for_sale,
            commands::invoices::invoices_generate_for_sale,
            commands::invoices::invoices_list,
            commands::invoices::invoices_open_folder,
            commands::settings::settings_get,
            commands::settings::settings_update,
            commands::dashboard::dashboard_stats,
            commands::stats::stats_period,
            commands::stats::sales_history_30days,
            commands::backup::backup_create,
            commands::backup::backup_list,
            commands::backup::backup_restore,
            commands::backup::backup_get_db_path,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de l'application");
}
