import { useEffect, useState } from "react";
import { Package, Boxes, AlertTriangle, XCircle, ShoppingCart, Wallet, TrendingUp, Tags, Truck } from "lucide-react";
import { api } from "../api/tauri";
import type { DashboardStats, Product } from "../types";

const periods = [
  { key: "today", label: "Aujourd'hui" },
  { key: "yesterday", label: "Hier" },
  { key: "7days", label: "7 derniers jours" },
];

function StatCard({ icon: Icon, label, value, tone = "brand" }: { icon: any; label: string; value: string; tone?: string }) {
  const tones: Record<string, string> = {
    brand: "bg-brand-50 text-brand-600",
    amber: "bg-amber-50 text-amber-600",
    red: "bg-red-50 text-red-600",
  };
  return (
    <div className="card p-5">
      <div className={`mb-3 inline-flex h-10 w-10 items-center justify-center rounded-lg ${tones[tone]}`}>
        <Icon size={20} />
      </div>
      <div className="text-2xl font-semibold text-gray-800">{value}</div>
      <div className="text-sm text-gray-400">{label}</div>
    </div>
  );
}

export default function Dashboard() {
  const [period, setPeriod] = useState("7days");
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [lowStock, setLowStock] = useState<Product[]>([]);
  const [currency, setCurrency] = useState("FCFA");

  useEffect(() => {
    api.settingsGet().then((s) => setCurrency(s.currency)).catch(() => {});
  }, []);

  useEffect(() => {
    api.dashboardStats(period).then(setStats).catch(() => {});
    api.productsLowStock().then(setLowStock).catch(() => {});
  }, [period]);

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-gray-800">Tableau de bord</h1>
          <p className="text-sm text-gray-400">Vision rapide de l'activité de la boutique</p>
        </div>
        <div className="flex gap-1 rounded-lg bg-gray-100 p-1">
          {periods.map((p) => (
            <button
              key={p.key}
              onClick={() => setPeriod(p.key)}
              className={`rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
                period === p.key ? "bg-white text-brand-600 shadow-sm" : "text-gray-500 hover:text-gray-700"
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>
      </div>

      {stats && (
        <>
          <div className="mb-4 grid grid-cols-2 gap-4 md:grid-cols-4">
            <StatCard icon={Package} label="Produits" value={String(stats.total_products)} />
            <StatCard icon={Boxes} label="Quantité en stock" value={String(stats.total_stock_qty)} />
            <StatCard icon={AlertTriangle} label="Proches de la rupture" value={String(stats.low_stock_count)} tone="amber" />
            <StatCard icon={XCircle} label="En rupture" value={String(stats.out_of_stock_count)} tone="red" />
          </div>
          <div className="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4">
            <StatCard icon={ShoppingCart} label="Ventes" value={String(stats.sales_count)} />
            <StatCard icon={Wallet} label="Montant des ventes" value={`${stats.sales_total.toLocaleString()} ${currency}`} />
            <StatCard icon={TrendingUp} label="Bénéfice estimé" value={`${stats.estimated_profit.toLocaleString()} ${currency}`} />
            <StatCard icon={Tags} label="Catégories" value={String(stats.categories_count)} />
          </div>
        </>
      )}

      <div className="card p-5">
        <div className="mb-3 flex items-center gap-2">
          <AlertTriangle size={18} className="text-amber-500" />
          <h2 className="font-semibold text-gray-800">Produits à surveiller</h2>
        </div>
        {lowStock.length === 0 ? (
          <p className="text-sm text-gray-400">Aucun produit proche de la rupture pour le moment.</p>
        ) : (
          <div className="divide-y divide-gray-100">
            {lowStock.map((p) => (
              <div key={p.id} className="flex items-center justify-between py-2.5">
                <div>
                  <div className="text-sm font-medium text-gray-700">{p.name}</div>
                  <div className="text-xs text-gray-400">{p.category_name}</div>
                </div>
                <div className={`text-sm font-semibold ${p.stock_qty === 0 ? "text-red-600" : "text-amber-600"}`}>
                  {p.stock_qty === 0 ? "Rupture" : `${p.stock_qty} restant(s)`}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
