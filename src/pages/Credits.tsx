import { useEffect, useState } from "react";
import { CreditCard, FileText, Search, Wallet } from "lucide-react";
import { api } from "../api/tauri";
import type { CreditAccount } from "../types";
import { useToast } from "../context/ToastContext";

export default function Credits() {
  const [accounts, setAccounts] = useState<CreditAccount[]>([]);
  const [selected, setSelected] = useState<CreditAccount | null>(null);
  const [selectedSaleId, setSelectedSaleId] = useState<number | null>(null);
  const [search, setSearch] = useState("");
  const [amount, setAmount] = useState("");
  const [note, setNote] = useState("");
  const [currency, setCurrency] = useState("FCFA");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const { showToast } = useToast();

  const money = (value: number) => `${value.toLocaleString("fr-FR")} ${currency}`;

  async function loadAccounts() {
    try {
      const data = await api.creditsList(search);
      setAccounts(data);
      if (selected) {
        const refreshed = await api.creditsGet(selected.customer.id);
        setSelected(refreshed.balance_due > 0 ? refreshed : null);
        if (refreshed.balance_due > 0 && !refreshed.sales.some((sale) => sale.sale_id === selectedSaleId && sale.balance_due > 0)) {
          setSelectedSaleId(refreshed.sales.find((sale) => sale.balance_due > 0)?.sale_id ?? null);
        }
      }
    } catch (e: any) {
      setError(e.message || "Impossible de charger les créances");
    }
  }

  useEffect(() => {
    api.settingsGet().then((settings) => setCurrency(settings.currency)).catch(() => {});
  }, []);

  useEffect(() => {
    const timer = setTimeout(loadAccounts, 200);
    return () => clearTimeout(timer);
  }, [search]);

  async function selectAccount(account: CreditAccount) {
    setError("");
    try {
      const accountDetails = await api.creditsGet(account.customer.id);
      setSelected(accountDetails);
      setSelectedSaleId(accountDetails.sales.find((sale) => sale.balance_due > 0)?.sale_id ?? null);
    } catch (e: any) {
      setError(e.message || "Impossible de charger le compte client");
    }
  }

  async function registerPayment() {
    if (!selected) return;
    if (!selectedSaleId) {
      setError("Sélectionnez la vente concernée par ce règlement.");
      return;
    }
    const value = Number(amount);
    if (!Number.isFinite(value) || value <= 0) {
      setError("Saisissez un montant de règlement valide.");
      return;
    }
    setSaving(true);
    setError("");
    try {
      const updated = await api.creditPaymentCreate(selected.customer.id, selectedSaleId, value, note);
      setSelected(updated);
      setSelectedSaleId(updated.sales.find((sale) => sale.balance_due > 0)?.sale_id ?? null);
      setAmount("");
      setNote("");
      await loadAccounts();
      showToast("Règlement enregistré. Le solde a été recalculé.");
    } catch (e: any) {
      setError(e.message || "Impossible d'enregistrer le règlement");
    } finally {
      setSaving(false);
    }
  }

  async function generateInvoice(saleId: number) {
    try {
      const invoice = await api.invoicesGenerateForSale(saleId, null);
      showToast(`Pro forma ${invoice.invoice_number} générée avec le nouveau solde.`);
    } catch (e: any) {
      showToast(e.message || "Impossible de générer la pro forma", "error");
    }
  }

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-gray-800">Créances clients</h1>
          <p className="text-sm text-gray-400">Suivi des ventes à crédit et des règlements</p>
        </div>
        <div className="rounded-lg bg-amber-50 px-4 py-2 text-right">
          <div className="text-xs text-amber-700">Clients débiteurs</div>
          <div className="text-lg font-semibold text-amber-800">{accounts.length}</div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(280px,0.8fr)_minmax(0,1.6fr)]">
        <section className="card overflow-hidden">
          <div className="border-b border-gray-100 p-4">
            <div className="relative">
              <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
              <input
                className="input w-full !pl-9"
                placeholder="Rechercher un client..."
                value={search}
                onChange={(event) => setSearch(event.target.value)}
              />
            </div>
          </div>
          <div className="max-h-[620px] divide-y divide-gray-100 overflow-y-auto">
            {accounts.map((account) => (
              <button
                key={account.customer.id}
                className={`w-full px-4 py-3 text-left transition-colors hover:bg-brand-50 ${
                  selected?.customer.id === account.customer.id ? "bg-brand-50" : ""
                }`}
                onClick={() => selectAccount(account)}
              >
                <div className="flex items-center justify-between gap-3">
                  <span className="font-medium text-gray-700">{account.customer.name}</span>
                  <span className="text-sm font-semibold text-red-600">{money(account.balance_due)}</span>
                </div>
                <div className="mt-1 text-xs text-gray-400">{account.customer.phone || "Téléphone non renseigné"}</div>
              </button>
            ))}
            {accounts.length === 0 && <p className="px-4 py-10 text-center text-sm text-gray-400">Aucune créance en cours</p>}
          </div>
        </section>

        <section className="space-y-6">
          {!selected ? (
            <div className="card flex min-h-[300px] flex-col items-center justify-center p-8 text-center">
              <CreditCard size={32} className="mb-3 text-gray-300" />
              <p className="font-medium text-gray-600">Sélectionnez un client</p>
              <p className="mt-1 text-sm text-gray-400">Vous verrez ici ses ventes, ses règlements et son solde.</p>
            </div>
          ) : (
            <>
              <div className="card p-5">
                <div className="flex flex-wrap items-start justify-between gap-4">
                  <div>
                    <h2 className="text-lg font-semibold text-gray-800">{selected.customer.name}</h2>
                    <p className="text-sm text-gray-400">{selected.customer.phone || "Téléphone non renseigné"}</p>
                  </div>
                  <div className="text-right">
                    <div className="text-xs uppercase text-gray-400">Solde restant</div>
                    <div className="text-2xl font-semibold text-red-600">{money(selected.balance_due)}</div>
                  </div>
                </div>
                <div className="mt-5 grid grid-cols-1 gap-3 sm:grid-cols-3">
                  <div className="rounded-lg bg-gray-50 p-3"><div className="text-xs text-gray-400">Total des ventes</div><div className="mt-1 font-semibold text-gray-700">{money(selected.total_sales)}</div></div>
                  <div className="rounded-lg bg-emerald-50 p-3"><div className="text-xs text-emerald-700">Total payé</div><div className="mt-1 font-semibold text-emerald-800">{money(selected.total_paid)}</div></div>
                  <div className="rounded-lg bg-red-50 p-3"><div className="text-xs text-red-700">Reste à payer</div><div className="mt-1 font-semibold text-red-800">{money(selected.balance_due)}</div></div>
                </div>
              </div>

              <div className="card p-5">
                <div className="mb-4 flex items-center gap-2"><Wallet size={18} className="text-brand-600" /><h2 className="font-semibold text-gray-800">Enregistrer un règlement</h2></div>
                <div className="grid grid-cols-1 gap-3 md:grid-cols-[1.4fr_1fr_1fr_auto] md:items-end">
                  <div>
                    <label className="label">Vente concernée</label>
                    <select className="input" value={selectedSaleId ?? ""} onChange={(event) => setSelectedSaleId(event.target.value ? Number(event.target.value) : null)}>
                      <option value="">Sélectionner une dette</option>
                      {selected.sales.filter((sale) => sale.balance_due > 0).map((sale) => (
                        <option key={sale.sale_id} value={sale.sale_id}>{sale.sale_number} - reste {money(sale.balance_due)}</option>
                      ))}
                    </select>
                  </div>
                  <div><label className="label">Montant</label><input className="input" type="number" min="1" max={selected.sales.find((sale) => sale.sale_id === selectedSaleId)?.balance_due ?? selected.balance_due} value={amount} onChange={(event) => setAmount(event.target.value)} placeholder="Montant" /></div>
                  <div><label className="label">Note</label><input className="input" value={note} onChange={(event) => setNote(event.target.value)} placeholder="Espèces, mobile money..." /></div>
                  <button className="btn-primary justify-center" onClick={registerPayment} disabled={saving}>{saving ? "Enregistrement..." : "Enregistrer"}</button>
                </div>
                {error && <p className="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">{error}</p>}
              </div>

              <div className="card overflow-hidden">
                <div className="border-b border-gray-100 px-5 py-4"><h2 className="font-semibold text-gray-800">Ventes et solde par facture</h2><p className="mt-1 text-xs text-gray-400">Chaque règlement est maintenant rattaché à la dette sélectionnée.</p></div>
                <div className="divide-y divide-gray-100">
                  {selected.sales.map((sale) => (
                    <div key={sale.sale_id} className="flex flex-wrap items-center justify-between gap-3 px-5 py-4">
                      <div><div className="font-medium text-gray-700">{sale.sale_number}</div><div className="text-xs text-gray-400">{new Date(sale.created_at).toLocaleString("fr-FR")}</div></div>
                      <div className="text-right text-sm"><div className="text-gray-500">Vente : {money(sale.total)}</div><div className={sale.balance_due > 0 ? "font-semibold text-red-600" : "font-semibold text-emerald-600"}>{sale.balance_due > 0 ? `Reste : ${money(sale.balance_due)}` : "Soldée"}</div></div>
                      <button title="Générer une nouvelle pro forma" className="btn-secondary !px-3" onClick={() => generateInvoice(sale.sale_id)}><FileText size={15} /> Pro forma</button>
                    </div>
                  ))}
                </div>
              </div>

              <div className="card overflow-hidden">
                <div className="border-b border-gray-100 px-5 py-4"><h2 className="font-semibold text-gray-800">Historique des règlements</h2></div>
                <div className="divide-y divide-gray-100">
                  {selected.payments.map((payment) => <div key={payment.id} className="flex items-center justify-between px-5 py-3 text-sm"><span className="text-gray-500">{new Date(payment.created_at).toLocaleString("fr-FR")} {payment.sale_id ? `· ${selected.sales.find((sale) => sale.sale_id === payment.sale_id)?.sale_number ?? "Vente"}` : "· Ancien règlement"} {payment.note ? `· ${payment.note}` : ""}</span><span className="font-semibold text-emerald-600">+ {money(payment.amount)}</span></div>)}
                  {selected.payments.length === 0 && <p className="px-5 py-6 text-sm text-gray-400">Aucun règlement enregistré pour ce client.</p>}
                </div>
              </div>
            </>
          )}
        </section>
      </div>
    </div>
  );
}
