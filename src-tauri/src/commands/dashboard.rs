use crate::db::Db;
use crate::models::DashboardStats;
use tauri::State;

/// period: "today" | "yesterday" | "7days" | "all"
#[tauri::command]
pub fn dashboard_stats(db: State<Db>, period: Option<String>) -> Result<DashboardStats, String> {
    let conn = db.0.lock().unwrap();
    let period = period.unwrap_or_else(|| "7days".into());

    let date_filter = match period.as_str() {
        "today" => "date(s.created_at) = date('now')".to_string(),
        "yesterday" => "date(s.created_at) = date('now', '-1 day')".to_string(),
        "7days" => "date(s.created_at) >= date('now', '-7 day')".to_string(),
        _ => "1=1".to_string(),
    };

    let total_products: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0)).unwrap_or(0);
    let total_stock_qty: i64 = conn.query_row("SELECT COALESCE(SUM(stock_qty),0) FROM products", [], |r| r.get(0)).unwrap_or(0);
    let low_stock_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM products WHERE stock_qty <= alert_threshold AND stock_qty > 0", [], |r| r.get(0))
        .unwrap_or(0);
    let out_of_stock_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM products WHERE stock_qty = 0", [], |r| r.get(0))
        .unwrap_or(0);
    let categories_count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0)).unwrap_or(0);
    let suppliers_count: i64 = conn.query_row("SELECT COUNT(*) FROM suppliers", [], |r| r.get(0)).unwrap_or(0);

    let sql_sales = format!(
        "SELECT COUNT(*), COALESCE(SUM(s.total),0) FROM sales s WHERE s.status = 'validee' AND {}",
        date_filter
    );
    let (sales_count, sales_total): (i64, f64) = conn
        .query_row(&sql_sales, [], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap_or((0, 0.0));

    let sql_profit = format!(
        "SELECT COALESCE(SUM((si.unit_price - si.purchase_price_at_sale) * si.quantity), 0)
         FROM sale_items si
         JOIN sales s ON s.id = si.sale_id
         WHERE s.status = 'validee' AND {}",
        date_filter
    );
    let estimated_profit: f64 = conn.query_row(&sql_profit, [], |r| r.get(0)).unwrap_or(0.0);

    Ok(DashboardStats {
        total_products,
        total_stock_qty,
        low_stock_count,
        out_of_stock_count,
        sales_count,
        sales_total,
        estimated_profit,
        categories_count,
        suppliers_count,
    })
}
