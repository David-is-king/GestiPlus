import { NavLink } from "react-router-dom";
import {
  LayoutDashboard, Package, Tags, ArrowLeftRight, ShoppingCart,
  FileText, Users, Truck, BarChart3, Settings, Store, LogOut, CreditCard,
} from "lucide-react";
import { useAuth } from "../store/auth";

const linkBase =
  "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors";
const linkActive = "bg-brand-500 text-white";
const linkInactive = "text-gray-600 hover:bg-brand-50 hover:text-brand-700";

function Item({ to, icon: Icon, label, end }: { to: string; icon: any; label: string; end?: boolean }) {
  return (
    <NavLink
      to={to}
      end={end}
      className={({ isActive }) => `${linkBase} ${isActive ? linkActive : linkInactive}`}
    >
      <Icon size={18} />
      <span>{label}</span>
    </NavLink>
  );
}

export default function Sidebar() {
  const { user, logout } = useAuth();

  return (
    <aside className="flex h-full w-64 shrink-0 flex-col border-r border-gray-200 bg-white">
      <div className="flex items-center gap-2 border-b border-gray-100 px-5 py-5">
        <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-brand-500 text-white">
          <Store size={18} />
        </div>
        <div>
          <div className="text-sm font-semibold text-gray-800">Gesti-Plus</div>
          <div className="text-xs text-gray-400">Gérez mieux. Vendez plus</div>
        </div>
      </div>

      <nav className="flex-1 space-y-1 overflow-y-auto px-3 py-4">
        <Item to="/" end icon={LayoutDashboard} label="Tableau de bord" />

        <div className="px-3 pb-1 pt-3 text-[11px] font-semibold uppercase tracking-wide text-gray-400">Stock</div>
        <Item to="/stock" end icon={Package} label="Produits" />
        <Item to="/stock/categories" icon={Tags} label="Catégories" />
        <Item to="/stock/mouvements" icon={ArrowLeftRight} label="Mouvements" />

        <div className="px-3 pb-1 pt-3 text-[11px] font-semibold uppercase tracking-wide text-gray-400">Activité</div>
        <Item to="/ventes" icon={ShoppingCart} label="Ventes" />
        <Item to="/factures" icon={FileText} label="Factures" />
        <Item to="/clients" icon={Users} label="Clients" />
        <Item to="/creances" icon={CreditCard} label="Créances" />
        <Item to="/fournisseurs" icon={Truck} label="Fournisseurs" />

        <div className="px-3 pb-1 pt-3 text-[11px] font-semibold uppercase tracking-wide text-gray-400">Pilotage</div>
        <Item to="/statistiques" icon={BarChart3} label="Statistiques" />
        <Item to="/parametres" icon={Settings} label="Paramètres" />
      </nav>

      <div className="border-t border-gray-100 p-3">
        <div className="mb-2 px-2 text-xs text-gray-400">Connecté en tant que</div>
        <div className="mb-3 flex items-center justify-between px-2">
          <span className="text-sm font-medium text-gray-700">{user?.username}</span>
        </div>
        <button onClick={logout} className="btn-secondary w-full">
          <LogOut size={16} /> Se déconnecter
        </button>
      </div>
    </aside>
  );
}
