import { useEffect, useState } from "react";
import { ArrowDownCircle, ArrowUpCircle, Wrench } from "lucide-react";
import { api } from "../../api/tauri";
import type { StockMovement } from "../../types";

const typeMeta: Record<string, { label: string; icon: any; color: string }> = {
  entree: { label: "Entrée", icon: ArrowDownCircle, color: "text-emerald-600 bg-emerald-50" },
  sortie: { label: "Sortie", icon: ArrowUpCircle, color: "text-red-600 bg-red-50" },
  ajustement: { label: "Ajustement", icon: Wrench, color: "text-amber-600 bg-amber-50" },
};

export default function Movements() {
  const [movements, setMovements] = useState<StockMovement[]>([]);
  const [filterType, setFilterType] = useState<string>("");

  useEffect(() => {
    api.movementsList(undefined, filterType || undefined).then(setMovements).catch(() => {});
  }, [filterType]);

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-gray-800">Mouvements de stock</h1>
          <p className="text-sm text-gray-400">Historique des entrées, sorties et ajustements</p>
        </div>
        <select className="input w-48" value={filterType} onChange={(e) => setFilterType(e.target.value)}>
          <option value="">Tous les types</option>
          <option value="entree">Entrées</option>
          <option value="sortie">Sorties</option>
          <option value="ajustement">Ajustements</option>
        </select>
      </div>

      <div className="card divide-y divide-gray-100">
        {movements.map((m) => {
          const meta = typeMeta[m.type];
          const Icon = meta.icon;
          return (
            <div key={m.id} className="flex items-center gap-3 px-4 py-3">
              <div className={`flex h-9 w-9 items-center justify-center rounded-lg ${meta.color}`}>
                <Icon size={18} />
              </div>
              <div className="flex-1">
                <div className="text-sm font-medium text-gray-700">{m.product_name}</div>
                <div className="text-xs text-gray-400">
                  {meta.label} · {m.motif ?? "—"} {m.reference ? `· Réf. ${m.reference}` : ""}
                </div>
              </div>
              <div className="text-right">
                <div className="text-sm font-semibold text-gray-700">
                  {m.stock_before} → {m.stock_after}
                </div>
                <div className="text-xs text-gray-400">{new Date(m.created_at).toLocaleString("fr-FR")}</div>
              </div>
            </div>
          );
        })}
        {movements.length === 0 && <p className="px-4 py-8 text-center text-gray-400">Aucun mouvement</p>}
      </div>
    </div>
  );
}
