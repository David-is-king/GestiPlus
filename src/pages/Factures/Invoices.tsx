import { useEffect, useState } from "react";
import { Search, FolderOpen } from "lucide-react";
import { api } from "../../api/tauri";
import type { Invoice } from "../../types";

export default function Invoices() {
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [search, setSearch] = useState("");
  const [currency, setCurrency] = useState("FCFA");

  useEffect(() => {
    api.settingsGet().then((s) => setCurrency(s.currency));
  }, []);

  useEffect(() => {
    const t = setTimeout(() => api.invoicesList(search).then(setInvoices), 200);
    return () => clearTimeout(t);
  }, [search]);

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-gray-800">Factures pro forma</h1>
        <p className="text-sm text-gray-400">Retrouvez et réimprimez les factures générées</p>
      </div>

      <div className="mb-4 relative w-full max-w-sm w-full flex items-center">
        <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
        <input className="input w-full !pl-9" placeholder="Rechercher une facture..." value={search} onChange={(e) => setSearch(e.target.value)} />
      </div>

      <div className="card overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-gray-50 text-left text-xs uppercase text-gray-400">
            <tr>
              <th className="px-4 py-3">N° facture</th>
              <th className="px-4 py-3">Client</th>
              <th className="px-4 py-3">Total</th>
              <th className="px-4 py-3">Date</th>
              <th className="px-4 py-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {invoices.map((inv) => (
              <tr key={inv.id} className="hover:bg-gray-50">
                <td className="px-4 py-3 font-medium text-gray-700">{inv.invoice_number}</td>
                <td className="px-4 py-3 text-gray-500">{inv.customer_name ?? "Client comptoir"}</td>
                <td className="px-4 py-3 text-gray-500">{inv.total.toLocaleString()} {currency}</td>
                <td className="px-4 py-3 text-gray-400">{new Date(inv.created_at).toLocaleString("fr-FR")}</td>
                <td className="px-4 py-3 text-right">
                  <button
                    title="Ouvrir le dossier contenant le PDF"
                    className="rounded-md p-1.5 text-gray-400 hover:bg-brand-50 hover:text-brand-600"
                    onClick={() => api.invoicesOpenFolder(inv.id)}
                  >
                    <FolderOpen size={16} />
                  </button>
                </td>
              </tr>
            ))}
            {invoices.length === 0 && <tr><td colSpan={5} className="px-4 py-8 text-center text-gray-400">Aucune facture</td></tr>}
          </tbody>
        </table>
      </div>
    </div>
  );
}
