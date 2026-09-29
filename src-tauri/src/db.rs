use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);

pub fn app_data_dir() -> PathBuf {
    let mut dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("GestionBoutique");
    std::fs::create_dir_all(&dir).ok();
    dir
}

pub fn db_path() -> PathBuf {
    let mut p = app_data_dir();
    p.push("boutique.db");
    p
}

pub fn documents_root() -> PathBuf {
    let mut dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("GestionBoutique");
    std::fs::create_dir_all(&dir).ok();
    dir
}

pub fn factures_dir(year: i32) -> PathBuf {
    let mut p = documents_root();
    p.push("Factures");
    p.push(year.to_string());
    std::fs::create_dir_all(&p).ok();
    p
}

pub fn backups_dir() -> PathBuf {
    let mut p = documents_root();
    p.push("Sauvegardes");
    std::fs::create_dir_all(&p).ok();
    p
}

pub fn init_connection() -> Connection {
    let conn = Connection::open(db_path()).expect("Impossible d'ouvrir la base de données");
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
        .expect("Erreur PRAGMA");
    run_migrations(&conn);
    seed_defaults(&conn);
    conn
}

fn run_migrations(conn: &Connection) {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS store_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            name TEXT NOT NULL DEFAULT 'Ma Boutique',
            emplacement TEXT,
            quartier TEXT,
            rue TEXT,
            phone1 TEXT,
            phone2 TEXT,
            phone3 TEXT,
            email TEXT,
            currency TEXT NOT NULL DEFAULT 'FCFA',
            slogan TEXT,
            logo_path TEXT
        );

        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            name_normalized TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS suppliers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            phone1 TEXT,
            phone2 TEXT,
            address TEXT,
            email TEXT,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS customers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            phone TEXT,
            address TEXT,
            email TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            name_normalized TEXT NOT NULL UNIQUE,
            category_id INTEGER NOT NULL REFERENCES categories(id),
            purchase_price REAL NOT NULL DEFAULT 0,
            sale_price REAL NOT NULL DEFAULT 0,
            stock_qty INTEGER NOT NULL DEFAULT 0,
            alert_threshold INTEGER NOT NULL DEFAULT 0,
            supplier_id INTEGER REFERENCES suppliers(id),
            supplier_ref TEXT,
            internal_ref TEXT,
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS stock_movements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL REFERENCES products(id),
            type TEXT NOT NULL CHECK (type IN ('entree','sortie','ajustement')),
            quantity INTEGER NOT NULL,
            stock_before INTEGER NOT NULL,
            stock_after INTEGER NOT NULL,
            motif TEXT,
            reference TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS sales (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sale_number TEXT NOT NULL UNIQUE,
            customer_id INTEGER REFERENCES customers(id),
            total REAL NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'validee' CHECK (status IN ('validee','annulee')),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            cancelled_at TEXT
        );

        CREATE TABLE IF NOT EXISTS sale_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sale_id INTEGER NOT NULL REFERENCES sales(id),
            product_id INTEGER NOT NULL REFERENCES products(id),
            product_name TEXT NOT NULL,
            unit_price REAL NOT NULL,
            purchase_price_at_sale REAL NOT NULL,
            quantity INTEGER NOT NULL,
            subtotal REAL NOT NULL
        );

        CREATE TABLE IF NOT EXISTS proforma_invoices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            invoice_number TEXT NOT NULL UNIQUE,
            sale_id INTEGER REFERENCES sales(id),
            customer_id INTEGER REFERENCES customers(id),
            total REAL NOT NULL DEFAULT 0,
            pdf_path TEXT,
            status TEXT NOT NULL DEFAULT 'proforma' CHECK (status IN ('proforma','validee')),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS proforma_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            invoice_id INTEGER NOT NULL REFERENCES proforma_invoices(id),
            product_name TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            unit_price REAL NOT NULL,
            subtotal REAL NOT NULL
        );

        CREATE TABLE IF NOT EXISTS invoice_counters (
            year INTEGER PRIMARY KEY,
            last_number INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_products_category ON products(category_id);
        CREATE INDEX IF NOT EXISTS idx_products_supplier ON products(supplier_id);
        CREATE INDEX IF NOT EXISTS idx_movements_product ON stock_movements(product_id);
        CREATE INDEX IF NOT EXISTS idx_sale_items_sale ON sale_items(sale_id);
        "#,
    )
    .expect("Erreur lors des migrations");
}

fn seed_defaults(conn: &Connection) {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
        .unwrap_or(0);
    if count == 0 {
        let hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST).unwrap();
        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            rusqlite::params!["admin", hash],
        )
        .ok();
    }

    let settings_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM store_settings", [], |r| r.get(0))
        .unwrap_or(0);
    if settings_count == 0 {
        conn.execute(
            "INSERT INTO store_settings (id, name, currency) VALUES (1, 'Ma Boutique', 'FCFA')",
            [],
        )
        .ok();
    }

    let cat_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
        .unwrap_or(0);
    if cat_count == 0 {
        for name in ["Outillage", "Plomberie", "Électricité", "Divers"] {
            conn.execute(
                "INSERT OR IGNORE INTO categories (name, name_normalized) VALUES (?1, ?2)",
                rusqlite::params![name, normalize(name)],
            )
            .ok();
        }
    }
}

/// Normalise un texte pour comparaison insensible à la casse et aux espaces superflus.
pub fn normalize(s: &str) -> String {
    s.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}
