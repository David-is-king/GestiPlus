use crate::db::{factures_dir, Db};
use crate::models::{Invoice, SaleItem, StoreSettings};
use crate::pdf::{generate_invoice_pdf, InvoiceData};
use chrono::Datelike;
use tauri::State;
use tauri_plugin_shell::ShellExt;

fn next_invoice_number(tx: &rusqlite::Connection, year: i32) -> Result<String, String> {
    tx.execute(
        "INSERT INTO invoice_counters (year, last_number) VALUES (?1, 1000)
         ON CONFLICT(year) DO NOTHING",
        [year],
    )
    .map_err(|e| e.to_string())?;

    // Compteur séparé des N° de vente : on utilise year+100000 comme espace de clés pour les
    // factures afin de ne jamais entrer en collision avec les numéros de vente V-YYYY-xxxxxx.
    let key = year + 100000;
    tx.execute(
        "INSERT INTO invoice_counters (year, last_number) VALUES (?1, 0) ON CONFLICT(year) DO NOTHING",
        [key],
    )
    .map_err(|e| e.to_string())?;
    tx.execute("UPDATE invoice_counters SET last_number = last_number + 1 WHERE year = ?1", [key])
        .map_err(|e| e.to_string())?;
    let n: i64 = tx
        .query_row("SELECT last_number FROM invoice_counters WHERE year = ?1", [key], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(format!("PF-{}-{:06}", year, n))
}

/// Génère une facture pro forma PDF pour une vente existante et l'enregistre dans
/// Documents/GestionBoutique/Factures/<année>/.
#[tauri::command]
pub fn invoices_generate_for_sale(
    db: State<Db>,
    sale_id: i64,
    custom_customer_name: Option<String>,
) -> Result<Invoice, String> {
    let conn = db.0.lock().unwrap();

    let (sale_number, customer_id, total, created_at): (String, Option<i64>, f64, String) = conn
        .query_row(
            "SELECT sale_number, customer_id, total, created_at FROM sales WHERE id = ?1",
            [sale_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|_| "Vente introuvable".to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, product_id, product_name, unit_price, quantity, subtotal FROM sale_items WHERE sale_id = ?1")
        .map_err(|e| e.to_string())?;
    let items: Vec<SaleItem> = stmt
        .query_map([sale_id], |r| {
            Ok(SaleItem {
                id: r.get(0)?,
                product_id: r.get(1)?,
                product_name: r.get(2)?,
                unit_price: r.get(3)?,
                quantity: r.get(4)?,
                subtotal: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    // Priorité au nom personnalisé saisi dans l'interface, sinon récupération en BDD via customer_id
    let (customer_name, customer_phone): (Option<String>, Option<String>) =
        if let Some(ref custom_name) = custom_customer_name {
            if !custom_name.trim().is_empty() {
                (Some(custom_name.trim().to_string()), None)
            } else {
                (None, None)
            }
        } else if let Some(cid) = customer_id {
            conn.query_row("SELECT name, phone FROM customers WHERE id = ?1", [cid], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap_or((None, None))
        } else {
            (None, None)
        };

    let settings: StoreSettings = conn
        .query_row(
            "SELECT name, emplacement, quartier, rue, phone1, phone2, phone3, email, currency, slogan, logo_path
             FROM store_settings WHERE id = 1",
            [],
            |r| {
                Ok(StoreSettings {
                    name: r.get(0)?, emplacement: r.get(1)?, quartier: r.get(2)?, rue: r.get(3)?,
                    phone1: r.get(4)?, phone2: r.get(5)?, phone3: r.get(6)?, email: r.get(7)?,
                    currency: r.get(8)?, slogan: r.get(9)?, logo_path: r.get(10)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let year = chrono::Local::now().year();
    let invoice_number = next_invoice_number(&conn, year)?;

    let dir = factures_dir(year);
    let mut path = dir.clone();
    path.push(format!("{}.pdf", invoice_number));

    let date_display = created_at.split(' ').next().unwrap_or(&created_at).to_string();

    generate_invoice_pdf(
        &settings,
        &InvoiceData {
            invoice_number: invoice_number.clone(),
            date: date_display,
            customer_name: customer_name.clone(),
            customer_phone,
            items: items.clone(),
            total,
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
             VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![invoice_id, item.product_name, item.quantity, item.unit_price, item.subtotal],
        )
        .map_err(|e| e.to_string())?;
    }

    let _ = sale_number;

    Ok(Invoice {
        id: invoice_id,
        invoice_number,
        sale_id: Some(sale_id),
        customer_id,
        customer_name,
        total,
        pdf_path: Some(pdf_path),
        status: "proforma".into(),
        created_at,
        items,
    })
}

#[tauri::command]
pub fn invoices_list(db: State<Db>, search: Option<String>) -> Result<Vec<Invoice>, String> {
    let conn = db.0.lock().unwrap();
    let q = search.unwrap_or_default().trim().to_lowercase();
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.invoice_number, i.sale_id, i.customer_id, c.name, i.total, i.pdf_path, i.status, i.created_at
             FROM proforma_invoices i LEFT JOIN customers c ON c.id = i.customer_id
             WHERE ?1 = '' OR lower(i.invoice_number) LIKE '%'||?1||'%' OR lower(coalesce(c.name,'')) LIKE '%'||?1||'%'
             ORDER BY i.created_at DESC LIMIT 500",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([q], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, f64>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for row in rows.filter_map(|r| r.ok()) {
        let (id, invoice_number, sale_id, customer_id, customer_name, total, pdf_path, status, created_at) = row;
        let mut istmt = conn
            .prepare("SELECT id, 0, product_name, unit_price, quantity, subtotal FROM proforma_items WHERE invoice_id = ?1")
            .map_err(|e| e.to_string())?;
        let items: Vec<SaleItem> = istmt
            .query_map([id], |r| {
                Ok(SaleItem {
                    id: r.get(0)?,
                    product_id: r.get(1)?,
                    product_name: r.get(2)?,
                    unit_price: r.get(3)?,
                    quantity: r.get(4)?,
                    subtotal: r.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        out.push(Invoice { id, invoice_number, sale_id, customer_id, customer_name, total, pdf_path, status, created_at, items });
    }
    Ok(out)
}

#[tauri::command]
pub fn invoices_open_folder(app: tauri::AppHandle, invoice_id: i64, db: State<Db>) -> Result<(), String> {
    let path: Option<String> = {
        let conn = db.0.lock().unwrap();
        conn.query_row("SELECT pdf_path FROM proforma_invoices WHERE id = ?1", [invoice_id], |r| r.get(0))
            .ok()
    };
    if let Some(p) = path {
        let folder = std::path::Path::new(&p).parent().map(|x| x.to_path_buf());
        if let Some(folder) = folder {
            app.shell()
                .open(folder.to_string_lossy().to_string(), None)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Ouvre directement le fichier PDF de la dernière facture liée à une vente
#[tauri::command]
pub fn invoices_open_pdf_for_sale(app: tauri::AppHandle, sale_id: i64, db: State<Db>) -> Result<(), String> {
    let conn = db.0.lock().unwrap();
    let pdf_path: Option<String> = conn
        .query_row(
            "SELECT pdf_path FROM proforma_invoices WHERE sale_id = ?1 ORDER BY id DESC LIMIT 1",
            [sale_id],
            |r| r.get(0),
        )
        .ok();

    if let Some(path) = pdf_path {
        app.shell()
            .open(path, None)
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Aucune facture générée pour cette vente. Veuillez d'abord cliquer sur l'icône de facture pour la générer.".into())
    }
}