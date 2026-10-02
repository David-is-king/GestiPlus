use crate::db::{factures_dir, Db};
use crate::commands::credits::{customer_balance, sale_balance};
use crate::models::{Invoice, SaleItem, StoreSettings};
use crate::pdf::{generate_invoice_pdf, InvoiceData};
use chrono::Datelike;
use tauri::State;
use tauri_plugin_opener::OpenerExt;

fn next_invoice_number(conn: &rusqlite::Connection, year: i32) -> Result<String, String> {
    let key = year + 100_000;
    conn.execute(
        "INSERT INTO invoice_counters (year, last_number) VALUES (?1, 0) ON CONFLICT(year) DO NOTHING",
        [key],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE invoice_counters SET last_number = last_number + 1 WHERE year = ?1",
        [key],
    )
    .map_err(|e| e.to_string())?;
    let number = conn
        .query_row(
            "SELECT last_number FROM invoice_counters WHERE year = ?1",
            [key],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(format!("PF-{}-{:06}", year, number))
}

#[tauri::command]
pub fn invoices_generate_for_sale(
    db: State<Db>,
    sale_id: i64,
    custom_customer_name: Option<String>,
) -> Result<Invoice, String> {
    let conn = db.0.lock().unwrap();
    let (customer_id, total, created_at): (Option<i64>, f64, String) = conn
        .query_row(
            "SELECT customer_id, total, created_at FROM sales WHERE id = ?1",
            [sale_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Vente introuvable".to_string())?;
    let payment_state = sale_balance(&conn, sale_id)?;
    let account_balance = customer_id
        .map(|id| customer_balance(&conn, id))
        .transpose()?
        .unwrap_or(0.0);

    let mut items_statement = conn
        .prepare(
            "SELECT id, product_id, product_name, unit_price, quantity, subtotal
             FROM sale_items WHERE sale_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let items: Vec<SaleItem> = items_statement
        .query_map([sale_id], |row| {
            Ok(SaleItem {
                id: row.get(0)?,
                product_id: row.get(1)?,
                product_name: row.get(2)?,
                unit_price: row.get(3)?,
                quantity: row.get(4)?,
                subtotal: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();
    drop(items_statement);

    let (customer_name, customer_phone): (Option<String>, Option<String>) =
        if let Some(name) = custom_customer_name.filter(|name| !name.trim().is_empty()) {
            (Some(name.trim().to_string()), None)
        } else if let Some(id) = customer_id {
            conn.query_row("SELECT name, phone FROM customers WHERE id = ?1", [id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap_or((None, None))
        } else {
            (None, None)
        };

    let settings = conn
        .query_row(
            "SELECT name, emplacement, quartier, rue, phone1, phone2, phone3, email,
                    currency, slogan, commercial_name, logo_path
             FROM store_settings WHERE id = 1",
            [],
            |row| {
                Ok(StoreSettings {
                    name: row.get(0)?,
                    emplacement: row.get(1)?,
                    quartier: row.get(2)?,
                    rue: row.get(3)?,
                    phone1: row.get(4)?,
                    phone2: row.get(5)?,
                    phone3: row.get(6)?,
                    email: row.get(7)?,
                    currency: row.get(8)?,
                    slogan: row.get(9)?,
                    commercial_name: row.get(10)?,
                    logo_path: row.get(11)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let year = chrono::Local::now().year();
    let invoice_number = next_invoice_number(&conn, year)?;
    let mut path = factures_dir(year);
    path.push(format!("{}.pdf", invoice_number));

    let date = created_at.split(' ').next().unwrap_or(&created_at).to_string();
    generate_invoice_pdf(
        &settings,
        &InvoiceData {
            invoice_number: invoice_number.clone(),
            date,
            customer_name: customer_name.clone(),
            customer_phone,
            items: items.clone(),
            total,
            amount_paid: payment_state.amount_paid,
            balance_due: payment_state.balance_due,
            account_balance,
            is_proforma: true,
        },
        &path,
    )?;

    let pdf_path = path.to_string_lossy().to_string();
    conn.execute(
        "INSERT INTO proforma_invoices (invoice_number, sale_id, customer_id, total, pdf_path, status)
         VALUES (?1, ?2, ?3, ?4, ?5, 'proforma')",
        rusqlite::params![invoice_number, sale_id, customer_id, total, pdf_path],
    )
    .map_err(|e| e.to_string())?;
    let invoice_id = conn.last_insert_rowid();

    for item in &items {
        conn.execute(
            "INSERT INTO proforma_items (invoice_id, product_name, quantity, unit_price, subtotal)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![invoice_id, item.product_name, item.quantity, item.unit_price, item.subtotal],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(Invoice {
        id: invoice_id,
        invoice_number,
        sale_id: Some(sale_id),
        customer_id,
        customer_name,
        total,
        amount_paid: payment_state.amount_paid,
        balance_due: payment_state.balance_due,
        account_balance,
        pdf_path: Some(pdf_path),
        status: "proforma".into(),
        created_at,
        items,
    })
}

#[tauri::command]
pub fn invoices_list(db: State<Db>, search: Option<String>) -> Result<Vec<Invoice>, String> {
    let conn = db.0.lock().unwrap();
    let query = search.unwrap_or_default().trim().to_lowercase();
    let mut statement = conn
        .prepare(
            "SELECT i.id, i.invoice_number, i.sale_id, i.customer_id, c.name, i.total,
                    i.pdf_path, i.status, i.created_at
             FROM proforma_invoices i
             LEFT JOIN customers c ON c.id = i.customer_id
             WHERE ?1 = '' OR lower(i.invoice_number) LIKE '%' || ?1 || '%'
                OR lower(coalesce(c.name, '')) LIKE '%' || ?1 || '%'
             ORDER BY i.created_at DESC LIMIT 500",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([query], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut invoices = Vec::new();
    for row in rows.filter_map(Result::ok) {
        let (id, invoice_number, sale_id, customer_id, customer_name, total, pdf_path, status, created_at) = row;
        let (amount_paid, balance_due, account_balance) = if let Some(sale_id) = sale_id {
            let payment_state = sale_balance(&conn, sale_id)?;
            let account_balance = customer_id
                .map(|id| customer_balance(&conn, id))
                .transpose()?
                .unwrap_or(0.0);
            (payment_state.amount_paid, payment_state.balance_due, account_balance)
        } else {
            (0.0, 0.0, 0.0)
        };
        let mut item_statement = conn
            .prepare(
                "SELECT id, 0, product_name, unit_price, quantity, subtotal
                 FROM proforma_items WHERE invoice_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let items = item_statement
            .query_map([id], |item| {
                Ok(SaleItem {
                    id: item.get(0)?,
                    product_id: item.get(1)?,
                    product_name: item.get(2)?,
                    unit_price: item.get(3)?,
                    quantity: item.get(4)?,
                    subtotal: item.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
        invoices.push(Invoice {
            id,
            invoice_number,
            sale_id,
            customer_id,
            customer_name,
            total,
            amount_paid,
            balance_due,
            account_balance,
            pdf_path,
            status,
            created_at,
            items,
        });
    }
    Ok(invoices)
}

#[tauri::command]
pub fn invoices_open_folder(
    app: tauri::AppHandle,
    invoice_id: i64,
    db: State<Db>,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let path: Option<String> = conn
        .query_row(
            "SELECT pdf_path FROM proforma_invoices WHERE id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .ok();
    drop(conn);

    if let Some(path) = path {
        if let Some(folder) = std::path::Path::new(&path).parent() {
            app.opener()
                .open_path(folder.to_string_lossy().to_string(), None::<&str>)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn invoices_open_pdf_for_sale(
    app: tauri::AppHandle,
    sale_id: i64,
    db: State<Db>,
) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let pdf_path: Option<String> = conn
        .query_row(
            "SELECT pdf_path FROM proforma_invoices WHERE sale_id = ?1 ORDER BY id DESC LIMIT 1",
            [sale_id],
            |row| row.get(0),
        )
        .ok();
    drop(conn);

    if let Some(path) = pdf_path {
        app.opener()
            .open_path(path, None::<&str>)
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Aucune facture generee pour cette vente.".into())
    }
}
