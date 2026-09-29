import { useEffect, useState } from "react";
import { Search, Plus, Pencil, PackagePlus, ClipboardEdit } from "lucide-react";
import { api } from "../../api/tauri";
import type { Product, Category, Supplier, ProductInput } from "../../types";
import { useToast } from "../../context/ToastContext";

const emptyInput: ProductInput = {
  name: "", category_id: 0, purchase_price: 0, sale_price: 0,
  stock_qty: 0, alert_threshold: 0, supplier_id: null, supplier_ref: "", internal_ref: "", description: "",
};

export default function ProductsList() {
  const [products, setProducts] = useState<Product[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [search, setSearch] = useState("");
  const [modal, setModal] = useState<null | "create" | "edit">(null);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [form, setForm] = useState<ProductInput>(emptyInput);
  const [error, setError] = useState("");
  const [stockModal, setStockModal] = useState<Product | null>(null);
  const [stockQty, setStockQty] = useState(0);
  const [stockMotif, setStockMotif] = useState("");
  
  const { showToast } = useToast();

  function load() {
    api.productsList(search).then(setProducts).catch(() => {});
  }

  useEffect(() => {
    api.categoriesList().then(setCategories);
    api.suppliersList().then(setSuppliers);
  }, []);

  useEffect(() => {
    const t = setTimeout(load, 200);
    return () => clearTimeout(t);
  }, [search]);

  function openCreate() {
    setForm({ ...emptyInput, category_id: categories[0]?.id ?? 0 });
    setEditingId(null);
    setError("");
    setModal("create");
  }

  function openEdit(p: Product) {
    setForm({
      name: p.name, category_id: p.category_id, purchase_price: p.purchase_price,
      sale_price: p.sale_price, stock_qty: p.stock_qty, alert_threshold: p.alert_threshold,
      supplier_id: p.supplier_id ?? null, supplier_ref: p.supplier_ref ?? "",
      internal_ref: p.internal_ref ?? "", description: p.description ?? "",
    });
    setEditingId(p.id);
    setError("");
    setModal("edit");
  }

  async function handleSave() {
    try {
      if (modal === "create") {
        await api.productsCreate(form);
        showToast("Produit ajouté avec succès !");
      } else if (editingId) {
        await api.productsUpdate(editingId, form);
        showToast("Produit mis à jour avec succès !");
      }
      setModal(null);
      load();
    } catch (e: any) {
      setError(e.message);
    }
  }

  async function handleAddStock() {
    if (!stockModal) return;
    try {
      await api.productsAddStock(stockModal.id, stockQty, stockMotif || undefined);
      setStockModal(null);
      setStockQty(0);
      setStockMotif("");
      showToast("Stock mis à jour avec succès !");
      load();
    } catch (e: any) {
      setError(e.message);
    }
  }

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-gray-800">Produits</h1>
          <p className="text-sm text-gray-400">Catalogue et stock actuel</p>
        </div>
        <button className="btn-primary" onClick={openCreate}>
          <Plus size={16} /> Nouveau produit
        </button>
      </div>

      <div className="mb-4 relative w-full max-w-sm flex items-center">
        <Search size={16} className="absolute left-3 z-10 pointer-events-none text-gray-400" />
        <input
          className="input w-full !pl-10"
          placeholder="Rechercher un produit..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      <div className="card overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 text-left text-xs uppercase tracking-wide text-gray-400">
            <tr>
              <th className="px-4 py-3">Produit</th>
              <th className="px-4 py-3">Catégorie</th>
              <th className="px-4 py-3">Prix vente</th>
              <th className="px-4 py-3">Stock</th>
              <th className="px-4 py-3">Fournisseur</th>
              <th className="px-4 py-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {products.map((p) => (
              <tr key={p.id} className="hover:bg-gray-50">
                <td className="px-4 py-3 font-medium text-gray-700">{p.name}</td>
                <td className="px-4 py-3 text-gray-500">{p.category_name}</td>
                <td className="px-4 py-3 text-gray-500">{p.sale_price.toLocaleString()}</td>
                <td className="px-4 py-3">
                  <span
                    className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                      p.stock_qty === 0 ? "bg-red-50 text-red-600" : p.stock_qty <= p.alert_threshold ? "bg-amber-50 text-amber-600" : "bg-gray-100 text-gray-600"
                    }`}
                  >
                    {p.stock_qty}
                  </span>
                </td>
                <td className="px-4 py-3 text-gray-500">{p.supplier_name ?? "—"}</td>
                <td className="px-4 py-3">
                  <div className="flex justify-end gap-2">
                    <button title="Ajouter du stock" className="rounded-md p-1.5 text-gray-400 hover:bg-brand-50 hover:text-brand-600" onClick={() => setStockModal(p)}>
                      <PackagePlus size={16} />
                    </button>
                    <button title="Modifier" className="rounded-md p-1.5 text-gray-400 hover:bg-brand-50 hover:text-brand-600" onClick={() => openEdit(p)}>
                      <Pencil size={16} />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
            {products.length === 0 && (
              <tr><td colSpan={6} className="px-4 py-8 text-center text-gray-400">Aucun produit</td></tr>
            )}
          </tbody>
        </table>
      </div>

      {modal && (
        <div className="fixed inset-0 z-20 flex items-center justify-center bg-black/30 p-4">
          <div className="w-full max-w-lg rounded-xl bg-white p-6 shadow-xl">
            <h2 className="mb-4 text-lg font-semibold text-gray-800">
              {modal === "create" ? "Nouveau produit" : "Modifier le produit"}
            </h2>
            <div className="grid grid-cols-2 gap-4">
              <div className="col-span-2">
                <label className="label">Nom du produit</label>
                <input className="input" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
              </div>
              <div>
                <label className="label">Catégorie</label>
                <select className="input" value={form.category_id} onChange={(e) => setForm({ ...form, category_id: Number(e.target.value) })}>
                  {categories.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
                </select>
              </div>
              <div>
                <label className="label">Fournisseur</label>
                <select className="input" value={form.supplier_id ?? ""} onChange={(e) => setForm({ ...form, supplier_id: e.target.value ? Number(e.target.value) : null })}>
                  <option value="">—</option>
                  {suppliers.map((s) => <option key={s.id} value={s.id}>{s.name}</option>)}
                </select>
              </div>
              <div>
                <label className="label">Prix d'achat</label>
                <input type="number" className="input" value={form.purchase_price} onChange={(e) => setForm({ ...form, purchase_price: Number(e.target.value) })} />
              </div>
              <div>
                <label className="label">Prix de vente</label>
                <input type="number" className="input" value={form.sale_price} onChange={(e) => setForm({ ...form, sale_price: Number(e.target.value) })} />
              </div>
              {modal === "create" && (
                <div>
                  <label className="label">Quantité initiale</label>
                  <input type="number" className="input" value={form.stock_qty} onChange={(e) => setForm({ ...form, stock_qty: Number(e.target.value) })} />
                </div>
              )}
              <div>
                <label className="label">Seuil d'alerte</label>
                <input type="number" className="input" value={form.alert_threshold} onChange={(e) => setForm({ ...form, alert_threshold: Number(e.target.value) })} />
              </div>
              <div>
                <label className="label">Référence interne</label>
                <input className="input" value={form.internal_ref ?? ""} onChange={(e) => setForm({ ...form, internal_ref: e.target.value })} />
              </div>
              <div>
                <label className="label">Référence fournisseur</label>
                <input className="input" value={form.supplier_ref ?? ""} onChange={(e) => setForm({ ...form, supplier_ref: e.target.value })} />
              </div>
              <div className="col-span-2">
                <label className="label">Description / remarque</label>
                <textarea className="input" rows={2} value={form.description ?? ""} onChange={(e) => setForm({ ...form, description: e.target.value })} />
              </div>
            </div>

            {error && <p className="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">{error}</p>}

            <div className="mt-5 flex justify-end gap-2">
              <button className="btn-secondary" onClick={() => setModal(null)}>Annuler</button>
              <button className="btn-primary" onClick={handleSave}>Enregistrer</button>
            </div>
          </div>
        </div>
      )}

      {stockModal && (
        <div className="fixed inset-0 z-20 flex items-center justify-center bg-black/30 p-4">
          <div className="w-full max-w-sm rounded-xl bg-white p-6 shadow-xl">
            <div className="mb-4 flex items-center gap-2">
              <ClipboardEdit size={18} className="text-brand-600" />
              <h2 className="text-lg font-semibold text-gray-800">Ajouter du stock</h2>
            </div>
            <p className="mb-3 text-sm text-gray-500">{stockModal.name} — stock actuel : {stockModal.stock_qty}</p>
            <label className="label">Quantité à ajouter</label>
            <input type="number" className="input mb-3" value={stockQty} onChange={(e) => setStockQty(Number(e.target.value))} />
            <label className="label">Motif (optionnel)</label>
            <input className="input" value={stockMotif} onChange={(e) => setStockMotif(e.target.value)} placeholder="Réception fournisseur..." />
            {error && <p className="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">{error}</p>}
            <div className="mt-5 flex justify-end gap-2">
              <button className="btn-secondary" onClick={() => setStockModal(null)}>Annuler</button>
              <button className="btn-primary" onClick={handleAddStock}>Valider</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}