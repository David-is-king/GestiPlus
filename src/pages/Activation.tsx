import { useState } from "react";
import { KeyRound, Store } from "lucide-react";
import { api } from "../api/tauri";

export default function Activation({ onActivated }: { onActivated: () => void }) {
  const [key, setKey] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    setError("");
    setLoading(true);
    try {
      await api.activationActivate(key);
      onActivated();
    } catch (e: any) {
      setError(e.message || "Clé d'activation incorrecte");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-gradient-to-br from-brand-900 via-brand-700 to-brand-500 p-4">
      <div className="w-full max-w-md rounded-2xl bg-white p-8 shadow-xl">
        <div className="mb-7 flex flex-col items-center text-center">
          <div className="mb-3 flex h-12 w-12 items-center justify-center rounded-xl bg-brand-500 text-white">
            <Store size={24} />
          </div>
          <h1 className="text-xl font-semibold text-gray-800">Activation de GestiPlus</h1>
          <p className="mt-1 text-sm text-gray-400">Saisissez la clé fournie avec votre licence.</p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="label">Clé d'activation</label>
            <div className="relative">
              <KeyRound size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
              <input
                className="input w-full !pl-9 uppercase"
                value={key}
                onChange={(event) => setKey(event.target.value)}
                placeholder="GESTIPLUS-2026-X7K9-P4M2"
                autoFocus
                required
              />
            </div>
          </div>
          {error && <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-600">{error}</p>}
          <button type="submit" disabled={loading} className="btn-primary w-full justify-center">
            {loading ? "Vérification..." : "Activer GestiPlus"}
          </button>
        </form>
      </div>
    </div>
  );
}
