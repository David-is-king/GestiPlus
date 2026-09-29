import { useEffect, useState } from "react";
import { Save, Database, KeyRound, User } from "lucide-react";
import { api } from "../api/tauri";
import type { StoreSettings } from "../types";
import { useAuth } from "../store/auth";

export default function SettingsPage() {
  const { user, setUser } = useAuth();
  const [settings, setSettings] = useState<StoreSettings | null>(null);
  const [savedMsg, setSavedMsg] = useState("");
  const [backups, setBackups] = useState<string[]>([]);
  const [dbPath, setDbPath] = useState("");

  const [newUsername, setNewUsername] = useState(user?.username ?? "");
  const [oldPassword, setOldPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [accountMsg, setAccountMsg] = useState("");
  const [accountError, setAccountError] = useState("");

  function loadBackups() {
    api.backupList().then(setBackups).catch(() => {});
  }

  useEffect(() => {
    api.settingsGet().then(setSettings);
    api.backupGetDbPath().then(setDbPath);
    loadBackups();
  }, []);

  async function handleSaveSettings() {
    if (!settings) return;
    try {
      await api.settingsUpdate(settings);
      setSavedMsg("Paramètres enregistrés.");
      setTimeout(() => setSavedMsg(""), 2500);
    } catch (e: any) {
      alert(e.message);
    }
  }

  async function handleBackup() {
    try {
      const path = await api.backupCreate();
      loadBackups();
      alert(`Sauvegarde créée : ${path}`);
    } catch (e: any) {
      alert(e.message);
    }
  }

  async function handleRestore(path: string) {
    if (!confirm("Restaurer cette sauvegarde ? Redémarrez l'application après cette opération.")) return;
    try {
      await api.backupRestore(path);
      alert("Base restaurée. Merci de redémarrer l'application.");
    } catch (e: any) {
      alert(e.message);
    }
  }

  async function handleChangeUsername() {
    if (!user) return;
    setAccountError(""); setAccountMsg("");
    try {
      await api.changeUsername(user.id, newUsername);
      setUser({ ...user, username: newUsername });
      setAccountMsg("Nom d'utilisateur mis à jour.");
    } catch (e: any) {
      setAccountError(e.message);
    }
  }

  async function handleChangePassword() {
    if (!user) return;
    setAccountError(""); setAccountMsg("");
    try {
      await api.changePassword(user.id, oldPassword, newPassword);
      setOldPassword(""); setNewPassword("");
      setAccountMsg("Mot de passe mis à jour.");
    } catch (e: any) {
      setAccountError(e.message);
    }
  }

  if (!settings) return null;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold text-gray-800">Paramètres</h1>
        <p className="text-sm text-gray-400">Informations de la boutique, compte et sauvegarde</p>
      </div>

      <div className="card p-5">
        <h2 className="mb-4 font-semibold text-gray-800">Informations de la boutique</h2>
        <div className="grid grid-cols-2 gap-4">
          <div><label className="label">Nom de la boutique</label><input className="input" value={settings.name} onChange={(e) => setSettings({ ...settings, name: e.target.value })} /></div>
          <div><label className="label">Devise</label><input className="input" value={settings.currency} onChange={(e) => setSettings({ ...settings, currency: e.target.value })} /></div>
          <div><label className="label">Emplacement</label><input className="input" value={settings.emplacement ?? ""} onChange={(e) => setSettings({ ...settings, emplacement: e.target.value })} /></div>
          <div><label className="label">Quartier</label><input className="input" value={settings.quartier ?? ""} onChange={(e) => setSettings({ ...settings, quartier: e.target.value })} /></div>
          <div><label className="label">Rue</label><input className="input" value={settings.rue ?? ""} onChange={(e) => setSettings({ ...settings, rue: e.target.value })} /></div>
          <div><label className="label">Slogan</label><input className="input" value={settings.slogan ?? ""} onChange={(e) => setSettings({ ...settings, slogan: e.target.value })} /></div>
          <div><label className="label">Téléphone 1</label><input className="input" value={settings.phone1 ?? ""} onChange={(e) => setSettings({ ...settings, phone1: e.target.value })} /></div>
          <div><label className="label">Téléphone 2</label><input className="input" value={settings.phone2 ?? ""} onChange={(e) => setSettings({ ...settings, phone2: e.target.value })} /></div>
          <div><label className="label">Téléphone 3</label><input className="input" value={settings.phone3 ?? ""} onChange={(e) => setSettings({ ...settings, phone3: e.target.value })} /></div>
          <div><label className="label">E-mail</label><input className="input" value={settings.email ?? ""} onChange={(e) => setSettings({ ...settings, email: e.target.value })} /></div>
        </div>
        <div className="mt-4 flex items-center gap-3">
          <button className="btn-primary" onClick={handleSaveSettings}><Save size={16} /> Enregistrer</button>
          {savedMsg && <span className="text-sm text-brand-600">{savedMsg}</span>}
        </div>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="card p-5">
          <div className="mb-3 flex items-center gap-2"><User size={18} className="text-brand-600" /><h2 className="font-semibold text-gray-800">Compte utilisateur</h2></div>
          <label className="label">Nom d'utilisateur</label>
          <div className="mb-3 flex gap-2">
            <input className="input" value={newUsername} onChange={(e) => setNewUsername(e.target.value)} />
            <button className="btn-secondary shrink-0" onClick={handleChangeUsername}>Modifier</button>
          </div>
          <div className="mb-1 flex items-center gap-2 text-sm font-medium text-gray-600"><KeyRound size={14} /> Changer le mot de passe</div>
          <input type="password" className="input mb-2" placeholder="Mot de passe actuel" value={oldPassword} onChange={(e) => setOldPassword(e.target.value)} />
          <input type="password" className="input mb-2" placeholder="Nouveau mot de passe" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} />
          <button className="btn-secondary" onClick={handleChangePassword}>Mettre à jour le mot de passe</button>
          {accountMsg && <p className="mt-2 text-sm text-brand-600">{accountMsg}</p>}
          {accountError && <p className="mt-2 text-sm text-red-600">{accountError}</p>}
        </div>

        <div className="card p-5">
          <div className="mb-3 flex items-center gap-2"><Database size={18} className="text-brand-600" /><h2 className="font-semibold text-gray-800">Sauvegarde &amp; restauration</h2></div>
          <p className="mb-3 text-xs text-gray-400 break-all">Base actuelle : {dbPath}</p>
          <button className="btn-primary mb-4 w-full" onClick={handleBackup}>Créer une sauvegarde maintenant</button>
          <div className="max-h-40 space-y-1 overflow-y-auto">
            {backups.map((b) => (
              <div key={b} className="flex items-center justify-between rounded-lg bg-gray-50 px-3 py-2 text-xs">
                <span className="truncate text-gray-600">{b.split(/[\\/]/).pop()}</span>
                <button className="font-medium text-brand-600 hover:underline" onClick={() => handleRestore(b)}>Restaurer</button>
              </div>
            ))}
            {backups.length === 0 && <p className="text-xs text-gray-400">Aucune sauvegarde pour le moment.</p>}
          </div>
        </div>
      </div>
    </div>
  );
}
