use crate::db::Db;
use crate::models::{PeriodStats, ProductSalesAgg};
use tauri::State;
use serde::Serialize;

#[derive(Serialize)]
pub struct DailyProductSales {
    pub date: String,
    pub total_quantity: i64,
    pub total_revenue: f64,
}

/// period: "today" | "week" | "7days" | "month" | "semester" | "year" | "custom"
/// Pour "custom", fournir date_from et date_to (YYYY-MM-DD).
#[tauri::command]
pub fn sales_history_30days(
    db: State<Db>,
    product_id: Option<i64>,
) -> Result<Vec<DailyProductSales>, String> {
    let conn = db.0.lock().unwrap();

    // Filtre optionnel pour un produit spécifique
    let product_filter = match product_id {
        Some(id) => format!("AND si.product_id = {}", id),
        None => "".to_string(),
    };

    // Génère la suite des 30 derniers jours et joint les ventes correspondantes
    let query = format!(
        "WITH RECURSIVE dates(date) AS (
            SELECT date('now', '-29 day')
            UNION ALL
            SELECT date(date, '+1 day')
            FROM dates
            WHERE date < date('now')
        )
        SELECT 
            d.date,
            COALESCE(SUM(si.quantity), 0) as total_quantity,
            COALESCE(SUM(si.subtotal), 0.0) as total_revenue
        FROM dates d
        LEFT JOIN sales s ON date(s.created_at) = d.date AND s.status = 'validee'
        LEFT JOIN sale_items si ON si.sale_id = s.id {}
        GROUP BY d.date
        ORDER BY d.date ASC",
        product_filter
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let history = stmt
        .query_map([], |r| {
            Ok(DailyProductSales {
                date: r.get(0)?,
                total_quantity: r.get(1)?,
                total_revenue: r.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(history)
}

#[tauri::command]
pub fn stats_period(
    db: State<Db>,
    period: String,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<PeriodStats, String> {
    let conn = db.0.lock().unwrap();

    let (from_expr, to_expr): (String, String) = match period.as_str() {
        "today" => ("date('now')".into(), "date('now')".into()),
        "week" | "7days" => ("date('now', '-7 day')".into(), "date('now')".into()),
        "month" => ("date('now', 'start of month')".into(), "date('now')".into()),
        "semester" => ("date('now', '-6 month')".into(), "date('now')".into()),
        "year" => ("date('now', 'start of year')".into(), "date('now')".into()),
        "custom" => {
            let f = date_from.clone().ok_or("date_from requis pour une période personnalisée")?;
            let t = date_to.clone().ok_or("date_to requis pour une période personnalisée")?;
            (format!("date('{}')", f), format!("date('{}')", t))
        }
        _ => return Err("Période inconnue".into()),
    };

    let where_clause = format!(
        "s.status = 'validee' AND date(s.created_at) >= {} AND date(s.created_at) <= {}",
        from_expr, to_expr
    );

    let sales_count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM sales s WHERE {}", where_clause), [], |r| r.get(0))
        .unwrap_or(0);

    let sql_items = format!(
        "SELECT COALESCE(SUM(si.quantity),0), COALESCE(SUM(si.subtotal),0),
                COALESCE(SUM(si.purchase_price_at_sale * si.quantity),0),
                COUNT(DISTINCT si.product_id)
         FROM sale_items si JOIN sales s ON s.id = si.sale_id WHERE {}",
        where_clause
    );
    let (quantity_sold, revenue, cost, distinct_products_sold): (i64, f64, f64, i64) = conn
        .query_row(&sql_items, [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap_or((0, 0.0, 0.0, 0));

    let gross_profit = revenue - cost;
    let average_basket = if sales_count > 0 { revenue / sales_count as f64 } else { 0.0 };

    let sql_top = format!(
        "SELECT si.product_name, SUM(si.quantity) qty, SUM(si.subtotal) rev
         FROM sale_items si JOIN sales s ON s.id = si.sale_id
         WHERE {}
         GROUP BY si.product_name ORDER BY qty DESC LIMIT 5",
        where_clause
    );
    let mut stmt = conn.prepare(&sql_top).map_err(|e| e.to_string())?;
    let top_products: Vec<ProductSalesAgg> = stmt
        .query_map([], |r| {
            Ok(ProductSalesAgg { product_name: r.get(0)?, quantity_sold: r.get(1)?, revenue: r.get(2)? })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let sql_bottom = format!(
        "SELECT si.product_name, SUM(si.quantity) qty, SUM(si.subtotal) rev
         FROM sale_items si JOIN sales s ON s.id = si.sale_id
         WHERE {}
         GROUP BY si.product_name ORDER BY qty ASC LIMIT 5",
        where_clause
    );
    let mut stmt2 = conn.prepare(&sql_bottom).map_err(|e| e.to_string())?;
    let bottom_products: Vec<ProductSalesAgg> = stmt2
        .query_map([], |r| {
            Ok(ProductSalesAgg { product_name: r.get(0)?, quantity_sold: r.get(1)?, revenue: r.get(2)? })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(PeriodStats {
        sales_count,
        quantity_sold,
        revenue,
        cost,
        gross_profit,
        average_basket,
        distinct_products_sold,
        top_products,
        bottom_products,
    })
}
