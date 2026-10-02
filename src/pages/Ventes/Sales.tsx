import { useEffect, useState } from "react";
import { Search, Plus, Trash2, FileText, Ban, Eye } from "lucide-react";
import { api } from "../../api/tauri";
import type { Product, Customer, Sale, SaleItemInput } from "../../types";
import { useToast } from "../../context/ToastContext";

interface CartLine {
  product: Product;
  quantity: number;
}

type PaymentStatus = "payee" | "partielle" | "credit";

export default function Sales() {
  const [allProducts, setAllProducts] = useState<Product[]>([]);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [search, setSearch] = useState("");
  const [cart, setCart] = useState<CartLine[]>([]);
  const { showToast } = useToast();

  // Mode de saisie du client : 'select' (liste) ou 'custom' (champ texte libre)
  const [clientMode, setClientMode] = useState<"select" | "custom">("select");
  const [customerId, setCustomerId] = useState<number | "">("");
  const [customCustomerName, setCustomCustomerName] = useState("");
  const [customCustomerPhone, setCustomCustomerPhone] = useState("");

  // Paiement : payée intégralement / partielle / à crédit
  const [paymentStatus, setPaymentStatus] = useState<PaymentStatus>("payee");
  const [amountPaid, setAmountPaid] = useState<number>(0);

  const [error, setError] = useState("");
  const [sales, setSales] = useState<Sale[]>([]);
  const [currency, setCurrency] = useState("FCFA");
  const [lastSale, setLastSale] = useState<Sale | null>(null);

  function loadSales() {
    api.salesList().then(setSales).catch(() => {});
  }

  function loadProducts() {
    api.productsList("").then(setAllProducts).catch(() => {});
  }

  useEffect(() => {
    api.customersList().then(setCustomers).catch(() => {});
    api.settingsGet().then((s) => setCurrency(s.currency)).catch(() => {});
    loadProducts();
    loadSales();
  }, []);

  // Filtrage local dynamique de tous les produits
  const filteredProducts = allProducts.filter((p) =>
    p.name.toLowerCase().includes(search.toLowerCase()) ||
    (p.internal_ref && p.internal_ref.toLowerCase().includes(search.toLowerCase()))
  );

  function addToCart(p: Product) {
    if (p.stock_qty <= 0) {
      showToast("Produit en rupture de stock", "error");
      return;
    }

    setCart((c) => {
      const existing = c.find((l) => l.product.id === p.id);
      if (existing) {
        if (existing.quantity >= p.stock_qty) {
          showToast("Stock maximal atteint pour ce produit", "error");
          return c;
        }
        return c.map((l) =>
          l.product.id === p.id ? { ...l, quantity: l.quantity + 1 } : l
        );
      }
      return [...c, { product: p, quantity: 1 }];
    });
  }

  function updateQty(id: number, qty: number) {
    if (qty <= 0) {
      removeLine(id);
      return;
    }

    setCart((c) =>
      c.map((l) => {
        if (l.product.id === id) {
          const maxQty = Math.min(qty, l.product.stock_qty);
          if (maxQty < qty) {
            showToast("La quantité ne peut pas dépasser le stock disponible", "error");
          }
          return { ...l, quantity: maxQty };
        }
        return l;
      })
    );
  }

  function removeLine(id: number) {
    setCart((c) => c.filter((l) => l.product.id !== id));
  }

  const total = cart.reduce((sum, l) => sum + l.product.sale_price * l.quantity, 0);
  const needsIdentifiedCustomer = paymentStatus !== "payee";

  function resetClientAndPaymentState() {
    setCustomerId("");
    setCustomCustomerName("");
    setCustomCustomerPhone("");
    setPaymentStatus("payee");
    setAmountPaid(0);
  }

  // Valider la vente
  async function handleValidate() {
    setError("");
    if (cart.length === 0) {
      setError("Ajoutez au moins un produit à la vente");
      return;
    }

    const selectedCustomerId = clientMode === "select" && customerId !== "" ? Number(customerId) : null;
    const typedName = clientMode === "custom" ? customCustomerName.trim() : "";
    const typedPhone = clientMode === "custom" ? customCustomerPhone.trim() : "";

    // Un paiement partiel ou à crédit exige un client identifiable par téléphone :
    // soit choisi dans la liste, soit saisi avec nom + téléphone (créé/retrouvé côté serveur).
    if (needsIdentifiedCustomer && !selectedCustomerId) {
      if (clientMode !== "custom" || !typedName || !typedPhone) {
        setError(
          "Pour un paiement partiel ou à crédit, choisissez un client existant dans la liste, ou saisissez son nom ET son téléphone."
        );
        return;
      }
    }

    if (paymentStatus === "partielle" && (amountPaid <= 0 || amountPaid >= total)) {
      setError("Le montant versé doit être supérieur à 0 et inférieur au total de la vente.");
      return;
    }

    const items: SaleItemInput[] = cart.map((l) => ({
      product_id: l.product.id,
      quantity: l.quantity,
    }));

    try {
      // 1. Enregistre la vente (le backend retrouve/crée le client si nécessaire)
      const sale = await api.salesCreate({
        customerId: selectedCustomerId,
        customerName: typedName || null,
        customerPhone: typedPhone || null,
        paymentStatus,
        amountPaid: paymentStatus === "partielle" ? amountPaid : undefined,
        items,
      });

      // 2. Génère la facture avec le nom personnalisé si saisi (cas "payée" sans fiche client)
      const customName = clientMode === "custom" && typedName ? typedName : null;
      await api.invoicesGenerateForSale(sale.id, customName);

      setLastSale(sale);
      setCart([]);
      resetClientAndPaymentState();
      loadSales();
      loadProducts(); // Rafraîchit les stocks des produits
      showToast("Vente ajoutée avec succès !");
    } catch (e: any) {
      setError(e.message || "Erreur lors de l'enregistrement de la vente");
    }
  }

  // Visualiser le PDF de la facture
  async function handleViewPdf(saleId: number) {
    try {
      await api.invoicesOpenPdfForSale(saleId);
    } catch (e: any) {
      showToast(e.message || "Impossible d'ouvrir le document", "error");
    }
  }

  // Annuler une vente
  async function handleCancel(saleId: number) {
    if (!confirm("Annuler cette vente ? Le stock sera rétabli.")) return;
    try {
      await api.salesCancel(saleId);
      showToast("Facture / Vente annulée avec succès !");
      loadSales();
      loadProducts();
    } catch (e: any) {
      showToast(e.message || "Erreur lors de l'annulation", "error");
    }
  }

  // Générer une facture
  async function handleInvoice(saleId: number) {
    try {
      const inv = await api.invoicesGenerateForSale(saleId, null);
      showToast(`Facture ${inv.invoice_number || ''} générée avec succès !`);
      loadSales();
    } catch (e: any) {
      showToast(e.message || "Erreur lors de la génération de la facture", "error");
    }
  }

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-gray-800">Ventes</h1>
        <p className="text-sm text-gray-400">Enregistrer une nouvelle vente</p>
      </div>

      <div className="mb-8 grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* SECTION GAUCHE : RECHERCHE ET SELECTION DE PRODUITS */}
        <div className="lg:col-span-2 card p-5 space-y-4">
          <div className="relative w-full">
            <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 z-10" />
            <input
              className="input w-full !pl-9"
              placeholder="Rechercher un produit à ajouter..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>

          {/* LISTE DES PRODUITS DISPONIBLES */}
          <div className="max-h-72 overflow-y-auto rounded-lg border border-gray-100 divide-y divide-gray-100">
            {filteredProducts.map((p) => (
              <div
                key={p.id}
                className="flex items-center justify-between p-3 hover:bg-gray-50 transition-colors"
              >
                <div>
                  <p className="font-medium text-gray-700 text-sm">{p.name}</p>
                  <p className="text-xs text-gray-400">
                    P.U : {p.sale_price.toLocaleString()} {currency} | Stock :{" "}
                    <span className={p.stock_qty === 0 ? "text-red-500 font-semibold" : "font-semibold"}>
                      {p.stock_qty}
                    </span>
                  </p>
                </div>
                <button
                  onClick={() => addToCart(p)}
                  disabled={p.stock_qty <= 0}
                  className="btn-secondary !py-1 !px-3 text-xs flex items-center gap-1 disabled:opacity-40"
                >
                  <Plus size={14} /> Ajouter
                </button>
              </div>
            ))}

            {filteredProducts.length === 0 && (
              <p className="py-8 text-center text-sm text-gray-400">
                Aucun produit ne correspond à votre recherche.
              </p>
            )}
          </div>

          {/* TABLEAU DU PANIER COURANT */}
          {cart.length > 0 && (
            <div className="pt-4 border-t border-gray-100">
              <h3 className="text-sm font-semibold text-gray-700 mb-2">Produits sélectionnés</h3>
              <table className="w-full text-sm">
                <thead className="text-left text-xs uppercase text-gray-400">
                  <tr>
                    <th className="pb-2">Produit</th>
                    <th className="pb-2">Qté</th>
                    <th className="pb-2">P.U.</th>
                    <th className="pb-2">Sous-total</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-100">
                  {cart.map((l) => (
                    <tr key={l.product.id}>
                      <td className="py-2 font-medium text-gray-700">{l.product.name}</td>
                      <td className="py-2">
                        <input
                          type="number"
                          min={1}
                          max={l.product.stock_qty}
                          className="input w-20 !p-1 text-center text-sm"
                          value={l.quantity}
                          onChange={(e) => updateQty(l.product.id, Number(e.target.value))}
                        />
                      </td>
                      <td className="py-2 text-gray-500">{l.product.sale_price.toLocaleString()}</td>
                      <td className="py-2 font-medium text-gray-700">
                        {(l.product.sale_price * l.quantity).toLocaleString()}
                      </td>
                      <td className="py-2 text-right">
                        <button
                          className="text-gray-400 hover:text-red-600 p-1"
                          onClick={() => removeLine(l.product.id)}
                        >
                          <Trash2 size={16} />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* SECTION DROITE : RÉCAPITULATIF ET VALIDATION */}
        <div className="card p-5 flex flex-col justify-between">
          <div>
            <h2 className="mb-3 font-semibold text-gray-800">Récapitulatif</h2>

            <div className="mb-2 flex items-center justify-between">
              <label className="label mb-0">Client</label>
              <button
                type="button"
                className="text-xs text-brand-600 hover:underline font-medium"
                onClick={() => {
                  setClientMode(clientMode === "select" ? "custom" : "select");
                  setCustomerId("");
                  setCustomCustomerName("");
                  setCustomCustomerPhone("");
                }}
              >
                {clientMode === "select" ? "Saisir un nom" : "Choisir un client"}
              </button>
            </div>

            {clientMode === "select" ? (
              <select
                className="input mb-3 w-full"
                value={customerId}
                onChange={(e) => setCustomerId(e.target.value ? Number(e.target.value) : "")}
              >
                <option value="">Client comptoir</option>
                {customers.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                    {c.phone ? ` — ${c.phone}` : ""}
                  </option>
                ))}
              </select>
            ) : (
              <>
                <input
                  type="text"
                  className="input mb-2 w-full"
                  placeholder="Nom du client (ex: Jean Dupont)"
                  value={customCustomerName}
                  onChange={(e) => setCustomCustomerName(e.target.value)}
                />
                {needsIdentifiedCustomer && (
                  <input
                    type="text"
                    className="input mb-3 w-full"
                    placeholder="Téléphone (obligatoire pour un crédit)"
                    value={customCustomerPhone}
                    onChange={(e) => setCustomCustomerPhone(e.target.value)}
                  />
                )}
              </>
            )}

            {/* STATUT DU PAIEMENT */}
            <label className="label mb-2">Statut du paiement</label>
            <div className="mb-3 grid grid-cols-3 gap-1 rounded-lg bg-gray-100 p-1 text-xs">
              {([
                { key: "payee", label: "Payé" },
                { key: "partielle", label: "Partiel" },
                { key: "credit", label: "À crédit" },
              ] as { key: PaymentStatus; label: string }[]).map((o) => (
                <button
                  key={o.key}
                  type="button"
                  onClick={() => setPaymentStatus(o.key)}
                  className={`rounded-md px-2 py-1.5 font-medium transition-colors ${
                    paymentStatus === o.key ? "bg-white text-brand-600 shadow-sm" : "text-gray-500 hover:text-gray-700"
                  }`}
                >
                  {o.label}
                </button>
              ))}
            </div>

            {paymentStatus === "partielle" && (
              <div className="mb-3">
                <label className="label">Montant versé maintenant</label>
                <input
                  type="number"
                  className="input w-full"
                  value={amountPaid}
                  min={0}
                  max={total}
                  onChange={(e) => setAmountPaid(Number(e.target.value))}
                />
                <p className="mt-1 text-xs text-gray-400">
                  Reste à payer : {Math.max(total - amountPaid, 0).toLocaleString()} {currency}
                </p>
              </div>
            )}

            {needsIdentifiedCustomer && clientMode === "select" && customerId === "" && (
              <p className="mb-3 rounded-lg bg-amber-50 px-3 py-2 text-xs text-amber-700">
                Un paiement partiel ou à crédit nécessite un client identifié : choisissez-le
                dans la liste, ou passez en « Saisir un nom » et renseignez son téléphone.
              </p>
            )}

            <div className="mb-4 flex items-center justify-between border-t border-gray-100 pt-4">
              <span className="text-sm text-gray-500">Total général</span>
              <span className="text-xl font-semibold text-brand-600">
                {total.toLocaleString()} {currency}
              </span>
            </div>

            {error && (
              <p className="mb-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">
                {error}
              </p>
            )}

            <button
              className="btn-primary w-full justify-center disabled:opacity-50"
              onClick={handleValidate}
              disabled={cart.length === 0}
            >
              <Plus size={16} /> Enregistrer la vente
            </button>
          </div>

          {lastSale && (
            <div className="mt-4 rounded-lg bg-brand-50 p-3 text-sm text-brand-700">
              Vente {lastSale.sale_number} enregistrée ({lastSale.total.toLocaleString()} {currency}).
              {lastSale.payment_status !== "payee" && (
                <span className="mt-1 block font-medium text-amber-700">
                  {lastSale.payment_status === "credit"
                    ? `Entièrement à crédit : ${lastSale.total.toLocaleString()} ${currency} dû.`
                    : `Reste à payer : ${(lastSale.total - lastSale.amount_paid).toLocaleString()} ${currency}.`}
                </span>
              )}
              <button
                className="mt-2 block font-medium underline"
                onClick={() => handleInvoice(lastSale.id)}
              >
                Générer la facture pro forma
              </button>
            </div>
          )}
        </div>
      </div>

      {/* TABLEAU HISTORIQUE DES VENTES */}
      <h2 className="mb-3 font-semibold text-gray-800">Historique des ventes</h2>
      <div className="card overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 text-left text-xs uppercase text-gray-400">
            <tr>
              <th className="px-4 py-3">N° vente</th>
              <th className="px-4 py-3">Client</th>
              <th className="px-4 py-3">Total</th>
              <th className="px-4 py-3">Statut</th>
              <th className="px-4 py-3">Date</th>
              <th className="px-4 py-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {sales.map((s) => {
              const isCancelled = s.status === "annulee";

              return (
                <tr key={s.id} className="hover:bg-gray-50">
                  <td className="px-4 py-3 font-medium text-gray-700">{s.sale_number}</td>
                  <td className="px-4 py-3 text-gray-500">{s.customer_name ?? "Client comptoir"}</td>
                  <td className="px-4 py-3 text-gray-500">
                    {s.total.toLocaleString()} {currency}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex flex-col gap-1">
                      <span
                        className={`w-fit rounded-full px-2 py-0.5 text-xs font-medium ${
                          isCancelled ? "bg-red-50 text-red-600" : "bg-emerald-50 text-emerald-600"
                        }`}
                      >
                        {isCancelled ? "Annulée" : "Validée"}
                      </span>
                      {!isCancelled && s.payment_status !== "payee" && (
                        <span
                          className={`w-fit rounded-full px-2 py-0.5 text-xs font-medium ${
                            s.payment_status === "credit" ? "bg-red-50 text-red-600" : "bg-amber-50 text-amber-600"
                          }`}
                        >
                          {s.payment_status === "credit"
                            ? `Crédit : ${s.total.toLocaleString()} ${currency}`
                            : `Reste : ${(s.total - s.amount_paid).toLocaleString()} ${currency}`}
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {new Date(s.created_at).toLocaleString("fr-FR")}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex justify-end gap-2">
                      {/* BOUTON VOIR (Toujours actif, même si annulée) */}
                      <button
                        title="Voir le PDF"
                        className="rounded-md p-1.5 text-gray-400 hover:bg-brand-50 hover:text-brand-600"
                        onClick={() => handleViewPdf(s.id)}
                      >
                        <Eye size={16} />
                      </button>

                      {/* BOUTON GÉNÉRER FACTURE (Désactivé / masqué si la vente est annulée) */}
                      {!isCancelled && (
                        <button
                          title="Générer / Imprimer facture"
                          className="rounded-md p-1.5 text-gray-400 hover:bg-brand-50 hover:text-brand-600"
                          onClick={() => handleInvoice(s.id)}
                        >
                          <FileText size={16} />
                        </button>
                      )}

                      {/* BOUTON ANNULER LA VENTE (Uniquement si pas encore annulée) */}
                      {!isCancelled && (
                        <button
                          title="Annuler la vente"
                          className="rounded-md p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600"
                          onClick={() => handleCancel(s.id)}
                        >
                          <Ban size={16} />
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              );
            })}
            {sales.length === 0 && (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-gray-400">
                  Aucune vente
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
