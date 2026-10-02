use crate::db::Db;
use crate::models::{CreditAccount, CreditPayment, CreditSale, Customer};
use tauri::State;

#[derive(Debug)]
pub struct SaleBalance {
    pub amount_paid: f64,
    pub balance_due: f64,
}

struct SaleRow {
    id: i64,
    sale_number: String,
    total: f64,
    initial_paid: f64,
    direct_paid: f64,
    legacy_paid: f64,
    created_at: String,
}

fn customer_from_row(row: &rusqlite::Row) -> rusqlite::Result<Customer> {
    Ok(Customer {
        id: row.get(0)?,
        name: row.get(1)?,
        phone: row.get(2)?,
        address: row.get(3)?,
        email: row.get(4)?,
    })
}

fn sale_rows(conn: &rusqlite::Connection, customer_id: i64) -> Result<Vec<SaleRow>, String> {
    let legacy_total: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM debt_payments
             WHERE customer_id = ?1 AND sale_id IS NULL",
            [customer_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut statement = conn
        .prepare(
            "SELECT s.id, s.sale_number, s.total, s.amount_paid,
                    s.created_at, COALESCE(SUM(p.amount), 0)
             FROM sales s
             LEFT JOIN debt_payments p ON p.sale_id = s.id
             WHERE s.customer_id = ?1 AND s.status = 'validee'
             GROUP BY s.id
             ORDER BY s.created_at ASC, s.id ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([customer_id], |row| {
            Ok(SaleRow {
                id: row.get(0)?,
                sale_number: row.get(1)?,
                total: row.get(2)?,
                initial_paid: row.get(3)?,
                direct_paid: row.get(5)?,
                legacy_paid: 0.0,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(statement);

    let mut remaining_legacy = legacy_total;
    let mut rows = rows;
    for sale in &mut rows {
        let direct_due = (sale.total - sale.initial_paid - sale.direct_paid).max(0.0);
        sale.legacy_paid = direct_due.min(remaining_legacy);
        remaining_legacy -= sale.legacy_paid;
    }
    Ok(rows)
}

fn row_balance(row: &SaleRow) -> SaleBalance {
    let amount_paid = row.initial_paid + row.direct_paid + row.legacy_paid;
    SaleBalance {
        amount_paid,
        balance_due: (row.total - amount_paid).max(0.0),
    }
}

pub fn sale_balance(conn: &rusqlite::Connection, sale_id: i64) -> Result<SaleBalance, String> {
    let customer_id: Option<i64> = conn
        .query_row("SELECT customer_id FROM sales WHERE id = ?1", [sale_id], |row| row.get(0))
        .map_err(|_| "Vente introuvable".to_string())?;
    let Some(customer_id) = customer_id else {
        let (total, initial_paid): (f64, f64) = conn
            .query_row("SELECT total, amount_paid FROM sales WHERE id = ?1", [sale_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;
        return Ok(SaleBalance { amount_paid: initial_paid, balance_due: (total - initial_paid).max(0.0) });
    };

    sale_rows(conn, customer_id)?
        .into_iter()
        .find(|row| row.id == sale_id)
        .map(|row| row_balance(&row))
        .ok_or_else(|| "Vente introuvable".into())
}

pub fn customer_balance(conn: &rusqlite::Connection, customer_id: i64) -> Result<f64, String> {
    Ok(sale_rows(conn, customer_id)?.iter().map(|row| row_balance(row).balance_due).sum())
}

pub fn load_account(conn: &rusqlite::Connection, customer_id: i64) -> Result<CreditAccount, String> {
    let customer = conn
        .query_row(
            "SELECT id, name, phone, address, email FROM customers WHERE id = ?1",
            [customer_id],
            customer_from_row,
        )
        .map_err(|_| "Client introuvable".to_string())?;
    let rows = sale_rows(conn, customer_id)?;
    let total_sales = rows.iter().map(|row| row.total).sum();
    let sales = rows
        .iter()
        .map(|row| {
            let balance = row_balance(row);
            CreditSale {
                sale_id: row.id,
                sale_number: row.sale_number.clone(),
                total: row.total,
                initial_paid: row.initial_paid,
                payments_applied: row.direct_paid + row.legacy_paid,
                amount_paid: balance.amount_paid,
                balance_due: balance.balance_due,
                created_at: row.created_at.clone(),
            }
        })
        .collect::<Vec<_>>();

    let mut statement = conn
        .prepare(
            "SELECT id, customer_id, sale_id, amount, note, created_at FROM debt_payments
             WHERE customer_id = ?1 ORDER BY created_at DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;
    let payments = statement
        .query_map([customer_id], |row| {
            Ok(CreditPayment {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                sale_id: row.get(2)?,
                amount: row.get(3)?,
                note: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|row| row.ok())
        .collect::<Vec<_>>();
    let total_paid = rows.iter().map(|row| row.initial_paid).sum::<f64>()
        + payments.iter().map(|payment| payment.amount).sum::<f64>();
    let balance_due = sales.iter().map(|sale| sale.balance_due).sum();

    Ok(CreditAccount { customer, total_sales, total_paid, balance_due, sales, payments })
}

#[tauri::command]
pub fn credits_list(db: State<Db>, search: Option<String>) -> Result<Vec<CreditAccount>, String> {
    let conn = db.0.lock().unwrap();
    let query = search.unwrap_or_default().trim().to_lowercase();
    let mut statement = conn
        .prepare(
            "SELECT id FROM customers
             WHERE ?1 = '' OR lower(name) LIKE '%' || ?1 || '%'
                OR lower(coalesce(phone, '')) LIKE '%' || ?1 || '%'
             ORDER BY name COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;
    let ids = statement
        .query_map([query], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|row| row.ok())
        .collect::<Vec<_>>();
    drop(statement);

    ids.into_iter()
        .map(|id| load_account(&conn, id))
        .filter(|account| account.as_ref().map(|value| value.balance_due > 0.0).unwrap_or(false))
        .collect()
}

#[tauri::command]
pub fn credits_get(db: State<Db>, customer_id: i64) -> Result<CreditAccount, String> {
    let conn = db.0.lock().unwrap();
    load_account(&conn, customer_id)
}

#[tauri::command]
pub fn credit_payment_create(
    db: State<Db>,
    customer_id: i64,
    sale_id: i64,
    amount: f64,
    note: Option<String>,
) -> Result<CreditAccount, String> {
    if amount <= 0.0 {
        return Err("Le montant du reglement doit etre superieur a zero".into());
    }
    let conn = db.0.lock().unwrap();
    let sale_customer: Option<i64> = conn
        .query_row("SELECT customer_id FROM sales WHERE id = ?1 AND status = 'validee'", [sale_id], |row| row.get(0))
        .map_err(|_| "Vente introuvable ou annulee".to_string())?;
    if sale_customer != Some(customer_id) {
        return Err("Cette vente n'appartient pas a ce client".into());
    }
    let sale_due = sale_balance(&conn, sale_id)?.balance_due;
    if sale_due <= 0.0 {
        return Err("Cette vente est deja reglee".into());
    }
    if amount > sale_due + 0.000_001 {
        return Err(format!("Le reglement ne peut pas depasser le solde de cette vente ({})", sale_due));
    }
    conn.execute(
        "INSERT INTO debt_payments (customer_id, sale_id, amount, note) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![customer_id, sale_id, amount, note.filter(|value| !value.trim().is_empty())],
    )
    .map_err(|e| e.to_string())?;
    load_account(&conn, customer_id)
}
