use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Supplier {
    pub id: i64,
    pub name: String,
    pub phone1: Option<String>,
    pub phone2: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Customer {
    pub id: i64,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub category_name: Option<String>,
    pub purchase_price: f64,
    pub sale_price: f64,
    pub stock_qty: i64,
    pub alert_threshold: i64,
    pub supplier_id: Option<i64>,
    pub supplier_name: Option<String>,
    pub supplier_ref: Option<String>,
    pub internal_ref: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProductInput {
    pub name: String,
    pub category_id: i64,
    pub purchase_price: f64,
    pub sale_price: f64,
    pub stock_qty: i64,
    pub alert_threshold: i64,
    pub supplier_id: Option<i64>,
    pub supplier_ref: Option<String>,
    pub internal_ref: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockMovement {
    pub id: i64,
    pub product_id: i64,
    pub product_name: Option<String>,
    #[serde(rename = "type")]
    pub movement_type: String,
    pub quantity: i64,
    pub stock_before: i64,
    pub stock_after: i64,
    pub motif: Option<String>,
    pub reference: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaleItemInput {
    pub product_id: i64,
    pub quantity: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaleItem {
    pub id: i64,
    pub product_id: i64,
    pub product_name: String,
    pub unit_price: f64,
    pub quantity: i64,
    pub subtotal: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sale {
    pub id: i64,
    pub sale_number: String,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub total: f64,
    pub status: String,
    pub amount_paid: f64,
    pub payment_status: String,
    pub created_at: String,
    pub items: Vec<SaleItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreSettings {
    pub name: String,
    pub emplacement: Option<String>,
    pub quartier: Option<String>,
    pub rue: Option<String>,
    pub phone1: Option<String>,
    pub phone2: Option<String>,
    pub phone3: Option<String>,
    pub email: Option<String>,
    pub currency: String,
    pub slogan: Option<String>,
    pub commercial_name: Option<String>,
    pub logo_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardStats {
    pub total_products: i64,
    pub total_stock_qty: i64,
    pub low_stock_count: i64,
    pub out_of_stock_count: i64,
    pub sales_count: i64,
    pub sales_total: f64,
    pub estimated_profit: f64,
    pub categories_count: i64,
    pub suppliers_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductSalesAgg {
    pub product_name: String,
    pub quantity_sold: i64,
    pub revenue: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeriodStats {
    pub sales_count: i64,
    pub quantity_sold: i64,
    pub revenue: f64,
    pub cost: f64,
    pub gross_profit: f64,
    pub average_basket: f64,
    pub distinct_products_sold: i64,
    pub top_products: Vec<ProductSalesAgg>,
    pub bottom_products: Vec<ProductSalesAgg>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Invoice {
    pub id: i64,
    pub invoice_number: String,
    pub sale_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub total: f64,
    pub amount_paid: f64,
    pub balance_due: f64,
    pub account_balance: f64,
    pub pdf_path: Option<String>,
    pub status: String,
    pub created_at: String,
    pub items: Vec<SaleItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreditPayment {
    pub id: i64,
    pub customer_id: i64,
    pub sale_id: Option<i64>,
    pub amount: f64,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreditSale {
    pub sale_id: i64,
    pub sale_number: String,
    pub total: f64,
    pub initial_paid: f64,
    pub payments_applied: f64,
    pub amount_paid: f64,
    pub balance_due: f64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreditAccount {
    pub customer: Customer,
    pub total_sales: f64,
    pub total_paid: f64,
    pub balance_due: f64,
    pub sales: Vec<CreditSale>,
    pub payments: Vec<CreditPayment>,
}
