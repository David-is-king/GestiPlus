export interface UserInfo {
  id: number;
  username: string;
}

export interface Category {
  id: number;
  name: string;
  created_at: string;
}

export interface Supplier {
  id: number;
  name: string;
  phone1?: string | null;
  phone2?: string | null;
  address?: string | null;
  email?: string | null;
  notes?: string | null;
}

export interface Customer {
  id: number;
  name: string;
  phone?: string | null;
  address?: string | null;
  email?: string | null;
}

export interface Product {
  id: number;
  name: string;
  category_id: number;
  category_name?: string | null;
  purchase_price: number;
  sale_price: number;
  stock_qty: number;
  alert_threshold: number;
  supplier_id?: number | null;
  supplier_name?: string | null;
  supplier_ref?: string | null;
  internal_ref?: string | null;
  description?: string | null;
}

export interface ProductInput {
  name: string;
  category_id: number;
  purchase_price: number;
  sale_price: number;
  stock_qty: number;
  alert_threshold: number;
  supplier_id?: number | null;
  supplier_ref?: string | null;
  internal_ref?: string | null;
  description?: string | null;
}

export interface StockMovement {
  id: number;
  product_id: number;
  product_name?: string | null;
  type: "entree" | "sortie" | "ajustement";
  quantity: number;
  stock_before: number;
  stock_after: number;
  motif?: string | null;
  reference?: string | null;
  created_at: string;
}

export interface SaleItem {
  id: number;
  product_id: number;
  product_name: string;
  unit_price: number;
  quantity: number;
  subtotal: number;
}

export interface SaleItemInput {
  product_id: number;
  quantity: number;
}

export interface Sale {
  id: number;
  sale_number: string;
  customer_id?: number | null;
  customer_name?: string | null;
  total: number;
  status: "validee" | "annulee";
  amount_paid: number;
  payment_status: "payee" | "partielle" | "credit";
  created_at: string;
  items: SaleItem[];
}

export interface Invoice {
  id: number;
  invoice_number: string;
  sale_id?: number | null;
  customer_id?: number | null;
  customer_name?: string | null;
  total: number;
  amount_paid: number;
  balance_due: number;
  account_balance: number;
  pdf_path?: string | null;
  status: string;
  created_at: string;
  items: SaleItem[];
}

export interface CreditPayment {
  id: number;
  customer_id: number;
  sale_id?: number | null;
  amount: number;
  note?: string | null;
  created_at: string;
}

export interface CreditSale {
  sale_id: number;
  sale_number: string;
  total: number;
  initial_paid: number;
  payments_applied: number;
  amount_paid: number;
  balance_due: number;
  created_at: string;
}

export interface CreditAccount {
  customer: Customer;
  total_sales: number;
  total_paid: number;
  balance_due: number;
  sales: CreditSale[];
  payments: CreditPayment[];
}

export interface StoreSettings {
  name: string;
  emplacement?: string | null;
  quartier?: string | null;
  rue?: string | null;
  phone1?: string | null;
  phone2?: string | null;
  phone3?: string | null;
  email?: string | null;
  currency: string;
  slogan?: string | null;
  commercial_name?: string | null;
  logo_path?: string | null;
}

export interface DashboardStats {
  total_products: number;
  total_stock_qty: number;
  low_stock_count: number;
  out_of_stock_count: number;
  sales_count: number;
  sales_total: number;
  estimated_profit: number;
  categories_count: number;
  suppliers_count: number;
}

export interface ProductSalesAgg {
  product_name: string;
  quantity_sold: number;
  revenue: number;
}

export interface PeriodStats {
  sales_count: number;
  quantity_sold: number;
  revenue: number;
  cost: number;
  gross_profit: number;
  average_basket: number;
  distinct_products_sold: number;
  top_products: ProductSalesAgg[];
  bottom_products: ProductSalesAgg[];
}

export interface DailyProductSales {
  date: string;
  total_quantity: number;
  total_revenue: number;
}
