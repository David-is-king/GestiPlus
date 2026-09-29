import { useEffect, useState } from "react";
import { Search, Plus, Pencil } from "lucide-react";
import { api } from "../api/tauri";
import type { Supplier } from "../types";
import { useToast } from "../context/ToastContext";

const empty: Omit<Supplier, "id"> = { name: "", phone1: "", phone2: "", address: "", email: "", notes: "" };

export default function Suppliers() {
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [search, setSearch] = useState("");
  const [modal, setModal] = useState<null | "create" | "edit">(null);
  const [form, setForm] = useState<Omit<Supplier, "id">>(empty);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [error, setError] = useState("");
const {showToast} = useToast ();
  function load() {
    api.suppliersList(search).then(setSuppliers).catch(() => {});
  }
  useEffect(() => { const t = setTimeout(load, 200); return () => clearTimeout(t); }, [search]);

  function openCreate() { setForm(empty); setEditingId(null); setError(""); setModal("create"); }
  function openEdit(s: Supplier) {
    setForm({ name: s.name, phone1: s.phone1 ?? "", phone2: s.phone2 ?? "", address: s.address ?? "", email: s.email ?? "", notes: s.notes ?? "" });
    setEditingId(s.id); setError(""); setModal("edit");
  }

  async function handleSave() {
    try {
      if (modal === "create") {
        await api.suppliersCreate(form);
        showToast("Fournisseur ajouté avec succès !");
      }
      else if (editingId) {
        await api.suppliersUpdate(editingId, { id: editingId, ...form });
        showToast("Fournisseur modifé avec succès !");
      }
      setModal(null);
      load();
    } catch (e: any) {
      setError(e.message);
    }
  }

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-gray-800">Fournisseurs</h1>
          <p className="text-sm text-gray-400">Répertoire des fournisseurs de la boutique</p>
        </div>
        <button className="btn-primary" onClick={openCreate}><Plus size={16} /> Nouveau fournisseur</button>
      </div>

      <div className="mb-4 relative w-full max-w-sm w-full flex items-center">
        <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
        <input className="input  w-full !pl-9" placeholder="Rechercher (nom, téléphone)..." value={search} onChange={(e) => setSearch(e.target.value)} />
      </div>

      <div className="card divide-y divide-gray-100">
        {suppliers.map((s) => (
          <div key={s.id} className="flex items-center justify-between px-4 py-3">
            <div>
              <div className="text-sm font-medium text-gray-700">{s.name}</div>
              <div className="text-xs text-gray-400">{s.phone1} {s.address ? `· ${s.address}` : ""}</div>
            </div>
            <button className="rounded-md p-1.5 text-gray-400 hover:bg-brand-50 hover:text-brand-600" onClick={() => openEdit(s)}>
              <Pencil size={16} />
            </button>
          </div>
        ))}
        {suppliers.length === 0 && <p className="px-4 py-8 text-center text-gray-400">Aucun fournisseur</p>}
      </div>

      {modal && (
        <div className="fixed inset-0 z-20 flex items-center justify-center bg-black/30 p-4">
          <div className="w-full max-w-md rounded-xl bg-white p-6 shadow-xl">
            <h2 className="mb-4 text-lg font-semibold text-gray-800">{modal === "create" ? "Nouveau fournisseur" : "Modifier le fournisseur"}</h2>
            <div className="space-y-3">
              <div><label className="label">Nom</label><input className="input" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></div>
              <div className="grid grid-cols-2 gap-3">
                <div><label className="label">Téléphone 1</label><input className="input" value={form.phone1 ?? ""} onChange={(e) => setForm({ ...form, phone1: e.target.value })} /></div>
                <div><label className="label">Téléphone 2</label><input className="input" value={form.phone2 ?? ""} onChange={(e) => setForm({ ...form, phone2: e.target.value })} /></div>
              </div>
              <div><label className="label">Adresse</label><input className="input" value={form.address ?? ""} onChange={(e) => setForm({ ...form, address: e.target.value })} /></div>
              <div><label className="label">E-mail</label><input className="input" value={form.email ?? ""} onChange={(e) => setForm({ ...form, email: e.target.value })} /></div>
              <div><label className="label">Notes</label><textarea className="input" rows={2} value={form.notes ?? ""} onChange={(e) => setForm({ ...form, notes: e.target.value })} /></div>
            </div>
            {error && <p className="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">{error}</p>}
            <div className="mt-5 flex justify-end gap-2">
              <button className="btn-secondary" onClick={() => setModal(null)}>Annuler</button>
              <button className="btn-primary" onClick={handleSave}>Enregistrer</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
