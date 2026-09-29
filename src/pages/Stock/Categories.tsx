import { useEffect, useState } from "react";
import { Plus, Pencil } from "lucide-react";
import { api } from "../../api/tauri";
import type { Category } from "../../types";
import { useToast } from "../../context/ToastContext";

export default function Categories() {
  const [categories, setCategories] = useState<Category[]>([]);
  const [name, setName] = useState("");
  const [editing, setEditing] = useState<Category | null>(null);
  const [error, setError] = useState("");

  const { showToast } = useToast();

  function load() {
    api.categoriesList().then(setCategories).catch(() => {});
  }
  useEffect(load, []);

  async function handleCreate() {
    setError("");
    try {
      await api.categoriesCreate(name);
      setName("");
      load();
      showToast("Catégorie ajoutée avec succès !");
    } catch (e: any) {
      setError(e.message);
    }
  }

  async function handleUpdate() {
    if (!editing) return;
    setError("");
    try {
      await api.categoriesUpdate(editing.id, editing.name);
      setEditing(null);
      load();
      showToast("Catégorie mis à jour avec succès !");
    } catch (e: any) {
      setError(e.message);
    }
  }

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-gray-800">Catégories</h1>
        <p className="text-sm text-gray-400">Organisez vos produits par catégorie</p>
      </div>

      <div className="mb-4 flex max-w-md gap-2">
        <input className="input" placeholder="Nouvelle catégorie..." value={name} onChange={(e) => setName(e.target.value)} />
        <button className="btn-primary shrink-0" onClick={handleCreate}><Plus size={16} /> Ajouter</button>
      </div>
      {error && <p className="mb-4 max-w-md rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">{error}</p>}

      <div className="card divide-y divide-gray-100">
        {categories.map((c) => (
          <div key={c.id} className="flex items-center justify-between px-4 py-3">
            {editing?.id === c.id ? (
              <input className="input mr-2" value={editing.name} onChange={(e) => setEditing({ ...editing, name: e.target.value })} />
            ) : (
              <span className="text-sm font-medium text-gray-700">{c.name}</span>
            )}
            {editing?.id === c.id ? (
              <div className="flex gap-2">
                <button className="btn-secondary" onClick={() => setEditing(null)}>Annuler</button>
                <button className="btn-primary" onClick={handleUpdate}>Valider</button>
              </div>
            ) : (
              <button className="rounded-md p-1.5 text-gray-400 hover:bg-brand-50 hover:text-brand-600" onClick={() => setEditing(c)}>
                <Pencil size={16} />
              </button>
            )}
          </div>
        ))}
        {categories.length === 0 && <p className="px-4 py-8 text-center text-gray-400">Aucune catégorie</p>}
      </div>
    </div>
  );
}
