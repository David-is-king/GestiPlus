use crate::db::Db;
use crate::models::{PeriodStats, ProductSalesAgg};
use tauri::State;

#[tauri::command] // <-- Obligatoire
pub fn stats_period( // <-- Must be 'pub'
    db: State<Db>,
    period: String,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<PeriodStats, String> {
    // ...
}

#[tauri::command] // <-- N'oubliez pas d'ajouter la nouvelle fonction ici aussi
pub fn sales_history_30days(
    db: State<Db>,
    product_id: Option<i64>,
) -> Result<Vec<DailyProductSales>, String> {
    // ...
}