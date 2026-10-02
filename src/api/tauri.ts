import { invoke } from "@tauri-apps/api/core";
import type {
  UserInfo,
  Category,
  Supplier,
  Customer,
  Product,
  ProductInput,
  StockMovement,
  Sale,
  SaleItemInput,
  Invoice,
  CreditAccount,
  StoreSettings,
  DashboardStats,
  PeriodStats,
} from "../types";

/** Petite enveloppe autour de `invoke` qui uniformise les erreurs pour l'UI. */
async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw new Error(typeof e === "string" ? e : (e as any)?.message ?? "Erreur inconnue");
  }
}

export interface DailyProductSales {
  date: string;
  total_quantity: number;
  total_revenue: number;
}

export const api = {
  activationStatus: () => call<boolean>("activation_status"),
  activationActivate: (key: string) => call<void>("activation_activate", { key }),

  // Auth
  login: (username: string, password: string) =>
    call<UserInfo>("auth_login", { username, password }),
  changePassword: (user_id: number, old_password: string, new_password: string) =>
    call<void>("auth_change_password", { userId: user_id, oldPassword: old_password, newPassword: new_password }),
  changeUsername: (user_id: number, new_username: string) =>
    call<void>("auth_change_username", { userId: user_id, newUsername: new_username }),

  // Catégories
  categoriesList: () => call<Category[]>("categories_list"),
  categoriesCreate: (name: string) => call<number>("categories_create", { name }),
  categoriesUpdate: (id: number, name: string) => call<void>("categories_update", { id, name }),
  categoriesDelete: (id: number) => call<void>("categories_delete", { id }),

  // Produits
  productsList: (search?: string) => call<Product[]>("products_list", { search }),
  productsGet: (id: number) => call<Product>("products_get", { id }),
  productsCreate: (input: ProductInput) => call<number>("products_create", { input }),
  productsUpdate: (id: number, input: ProductInput) => call<void>("products_update", { id, input }),
  productsAddStock: (id: number, quantity: number, motif?: string) =>
    call<number>("products_add_stock", { id, quantity, motif }),
  productsAdjustStock: (id: number, new_quantity: number, motif?: string) =>
    call<void>("products_adjust_stock", { id, newQuantity: new_quantity, motif }),
  productsLowStock: () => call<Product[]>("products_low_stock"),

  // Mouvements
  movementsList: (product_id?: number, movement_type?: string) =>
    call<StockMovement[]>("movements_list", { productId: product_id, movementType: movement_type }),

  // Fournisseurs
  suppliersList: (search?: string) => call<Supplier[]>("suppliers_list", { search }),
  suppliersCreate: (supplier: Omit<Supplier, "id">) =>
    call<number>("suppliers_create", { supplier: { id: 0, ...supplier } }),
  suppliersUpdate: (id: number, supplier: Supplier) =>
    call<void>("suppliers_update", { id, supplier }),

  // Clients
  customersList: (search?: string) => call<Customer[]>("customers_list", { search }),
  customersCreate: (customer: Omit<Supplier, "id">) =>
    call<number>("customers_create", { customer: { id: 0, ...customer } }),
  customersUpdate: (id: number, customer: Customer) =>
    call<void>("customers_update", { id, customer }),

  // Créances clients
  creditsList: (search?: string) => call<CreditAccount[]>("credits_list", { search }),
  creditsGet: (customer_id: number) => call<CreditAccount>("credits_get", { customerId: customer_id }),
  creditPaymentCreate: (customer_id: number, sale_id: number, amount: number, note?: string) =>
    call<CreditAccount>("credit_payment_create", { customerId: customer_id, saleId: sale_id, amount, note }),

  // Ventes
  salesCreate: (input: {
    customerId: number | null;
    customerName: string | null;
    customerPhone: string | null;
    paymentStatus: "payee" | "partielle" | "credit";
    amountPaid?: number;
    items: SaleItemInput[];
  }) => call<Sale>("sales_create", input),
  salesList: (date_from?: string, date_to?: string, search?: string) =>
    call<Sale[]>("sales_list", { dateFrom: date_from, dateTo: date_to, search }),
  salesCancel: (sale_id: number) => call<void>("sales_cancel", { saleId: sale_id }),

  // Factures
  invoicesGenerateForSale: (sale_id: number, custom_customer_name?: string | null) =>
    call<Invoice>("invoices_generate_for_sale", { saleId: sale_id, customCustomerName: custom_customer_name }),
  invoicesList: (search?: string) => call<Invoice[]>("invoices_list", { search }),
  invoicesOpenFolder: (invoice_id: number) => call<void>("invoices_open_folder", { invoiceId: invoice_id }),
  invoicesOpenPdfForSale: (sale_id: number) => call<void>("invoices_open_pdf_for_sale", { saleId: sale_id }),

  // Paramètres
  settingsGet: () => call<StoreSettings>("settings_get"),
  settingsUpdate: (settings: StoreSettings) => call<void>("settings_update", { settings }),

  // Tableau de bord / statistiques
  dashboardStats: (period?: string) => call<DashboardStats>("dashboard_stats", { period }),
  statsPeriod: (period: string, date_from?: string, date_to?: string) =>
    call<PeriodStats>("stats_period", { period, dateFrom: date_from, dateTo: date_to }),
  salesHistory30Days: (product_id?: number | null) =>
    call<DailyProductSales[]>("sales_history_30days", { productId: product_id }),

  // Sauvegarde
  backupCreate: () => call<string>("backup_create"),
  backupList: () => call<string[]>("backup_list"),
  backupRestore: (backup_path: string) => call<void>("backup_restore", { backupPath: backup_path }),
  backupGetDbPath: () => call<string>("backup_get_db_path"),
};
