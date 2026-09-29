import { useEffect, useState } from "react";
import { api } from "../api/tauri";
import type { PeriodStats, DailyProductSales, Product } from "../types";
import SalesChart from "../components/SalesChart"; // Fichier créé à l'étape 3

const options = [
  { key: "today", label: "Aujourd'hui" },
  { key: "week", label: "7 derniers jours" },
  { key: "month", label: "Ce mois" },
  { key: "semester", label: "6 derniers mois" },
  { key: "year", label: "Cette année" },
];

export default function Stats() {
  const [period, setPeriod] = useState("month");
  const [stats, setStats] = useState<PeriodStats | null>(null);
  const [currency, setCurrency] = useState("FCFA");

  // États pour le graphique d'évolution
  const [products, setProducts] = useState<Product[]>([]);
  const [selectedProductId, setSelectedProductId] = useState<string>("all");
  const [historyData, setHistoryData] = useState<DailyProductSales[]>([]);
  const [historyError, setHistoryError] = useState<string | null>(null);

  useEffect(() => {
    api.settingsGet().then((s) => setCurrency(s.currency));
    api.productsList().then(setProducts).catch(() => {});
  }, []);

  useEffect(() => {
    api.statsPeriod(period).then(setStats).catch(() => {});
  }, [period]);

  // Chargement de l'historique sur 30 jours à chaque changement de produit
  useEffect(() => {
    const prodId = selectedProductId === "all" ? null : Number(selectedProductId);
    setHistoryError(null);
    api.salesHistory30Days(prodId)
      .then(setHistoryData)
      .catch((error) => setHistoryError(error instanceof Error ? error.message : "Impossible de charger l'historique des ventes."));
  }, [selectedProductId]);

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-gray-800">Statistiques</h1>
          <p className="text-sm text-gray-400">Analyse de l'activité commerciale</p>
        </div>
        <select className="input w-56" value={period} onChange={(e) => setPeriod(e.target.value)}>
          {options.map((o) => <option key={o.key} value={o.key}>{o.label}</option>)}
        </select>
      </div>

      {stats && (
        <>
          <div className="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4">
            <div className="card p-5"><div className="text-xs text-gray-400">Ventes</div><div className="text-2xl font-semibold text-gray-800">{stats.sales_count}</div></div>
            <div className="card p-5"><div className="text-xs text-gray-400">Chiffre d'affaires</div><div className="text-2xl font-semibold text-gray-800">{stats.revenue.toLocaleString()} {currency}</div></div>
            <div className="card p-5"><div className="text-xs text-gray-400">Bénéfice brut</div><div className="text-2xl font-semibold text-brand-600">{stats.gross_profit.toLocaleString()} {currency}</div></div>
            <div className="card p-5"><div className="text-xs text-gray-400">Panier moyen</div><div className="text-2xl font-semibold text-gray-800">{Math.round(stats.average_basket).toLocaleString()} {currency}</div></div>
          </div>


          <div className="grid grid-cols-2 gap-6">
            <div className="card p-5">
              <h2 className="mb-3 font-semibold text-gray-800">Produits les plus vendus</h2>
              <div className="space-y-2">
                {stats.top_products.map((p, i) => (
                  <div key={p.product_name} className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">{i + 1}. {p.product_name}</span>
                    <span className="font-medium text-gray-700">{p.quantity_sold} unités</span>
                  </div>
                ))}
                {stats.top_products.length === 0 && <p className="text-sm text-gray-400">Pas encore de données</p>}
              </div>
            </div>
            <div className="card p-5">
              <h2 className="mb-3 font-semibold text-gray-800">Produits les moins vendus</h2>
              <div className="space-y-2">
                {stats.bottom_products.map((p, i) => (
                  <div key={p.product_name} className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">{i + 1}. {p.product_name}</span>
                    <span className="font-medium text-gray-700">{p.quantity_sold} unités</span>
                  </div>
                ))}
                {stats.bottom_products.length === 0 && <p className="text-sm text-gray-400">Pas encore de données</p>}
              </div>
            </div>
          </div>
          {/* SECTION GRAPHIQUE D'ÉVOLUTION SUR 30 JOURS */}
          <div className="card mb-6 p-5">
            <div className="mb-4 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <h2 className="font-semibold text-gray-800">Évolution des ventes (30 derniers jours)</h2>
                <p className="text-xs text-gray-400">Quantités vendues jour par jour</p>
              </div>
              <select
                className="input sm:w-64"
                value={selectedProductId}
                onChange={(e) => setSelectedProductId(e.target.value)}
              >
                <option value="all">Tous les produits</option>
                {products.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
            </div>

            {historyError ? (
              <p className="py-10 text-center text-sm text-red-600">{historyError}</p>
            ) : (
              <SalesChart data={historyData} currency={currency} />
            )}
          </div>
        </>
      )}
    </div>
  );
}
