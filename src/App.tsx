import { useEffect, useState } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import Login from "./pages/Login";
import Dashboard from "./pages/Dashboard";
import ProductsList from "./pages/Stock/ProductsList";
import Categories from "./pages/Stock/Categories";
import Movements from "./pages/Stock/Movements";
import Sales from "./pages/Ventes/Sales";
import Invoices from "./pages/Factures/Invoices";
import Clients from "./pages/Clients";
import Suppliers from "./pages/Suppliers";
import Stats from "./pages/Stats";
import SettingsPage from "./pages/SettingsPage";
import Credits from "./pages/Credits";
import { useAuth } from "./store/auth";
import { ToastProvider } from "./context/ToastContext"; // Adaptez le chemin selon votre structure
import { api } from "./api/tauri";

export default function App() {
  const { user } = useAuth();
  const [ready, setReady] = useState(false);

  useEffect(() => {
    // Laisse le temps au backend Tauri d'initialiser la base au premier lancement.
    const t = setTimeout(() => setReady(true), 150);
    return () => clearTimeout(t);
  }, []);

  useEffect(() => {
    if (!user) return;

    let running = false;
    const createAutomaticBackup = async () => {
      if (running) return;
      running = true;
      try {
        await api.backupCreate();
      } catch {
        // Une sauvegarde manuelle reste disponible si le dossier est indisponible.
      } finally {
        running = false;
      }
    };

    createAutomaticBackup();
    const timer = window.setInterval(createAutomaticBackup, 15 * 60 * 1000);
    return () => window.clearInterval(timer);
  }, [user]);

  if (!ready) return null;

  return (
    <ToastProvider>
      {!user ? (
        <Login />
      ) : (
        <div className="flex h-screen w-screen overflow-hidden bg-gray-50">
          <Sidebar />
          <main className="flex-1 overflow-y-auto">
            <div className="mx-auto max-w-6xl px-8 py-8">
              <Routes>
                <Route path="/" element={<Dashboard />} />
                <Route path="/stock" element={<ProductsList />} />
                <Route path="/stock/categories" element={<Categories />} />
                <Route path="/stock/mouvements" element={<Movements />} />
                <Route path="/ventes" element={<Sales />} />
                <Route path="/factures" element={<Invoices />} />
                <Route path="/clients" element={<Clients />} />
                <Route path="/creances" element={<Credits />} />
                <Route path="/fournisseurs" element={<Suppliers />} />
                <Route path="/statistiques" element={<Stats />} />
                <Route path="/parametres" element={<SettingsPage />} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </div>
          </main>
        </div>
      )}
    </ToastProvider>
  );
}
