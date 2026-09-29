import React, { createContext, useContext, useState } from "react";
import { CheckCircle2, AlertCircle } from "lucide-react";

type ToastType = "success" | "error";

interface ToastContextType {
  showToast: (message: string, type?: ToastType) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [toast, setToast] = useState<{ message: string; type: ToastType } | null>(null);

  const showToast = (message: string, type: ToastType = "success") => {
    setToast({ message, type });
    setTimeout(() => {
      setToast(null);
    }, 3000); // Disparaît au bout de 3 secondes
  };

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}

      {/* BOÎTE DE DIALOGUE / TOAST REUTILISABLE EN HAUT CENTRÉ */}
      {toast && (
        <div className="fixed top-6 left-1/2 z-50 -translate-x-1/2 transform animate-bounce">
          <div className="flex flex-col items-center justify-center gap-2 rounded-2xl bg-white px-8 py-5 shadow-2xl border border-gray-100 min-w-[280px]">
            {toast.type === "success" ? (
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-green-500 text-white shadow-md">
                <CheckCircle2 size={30} strokeWidth={2.5} />
              </div>
            ) : (
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-red-500 text-white shadow-md">
                <AlertCircle size={30} strokeWidth={2.5} />
              </div>
            )}
            <p className="text-center font-semibold text-gray-800 text-base mt-1">
              {toast.message}
            </p>
          </div>
        </div>
      )}
    </ToastContext.Provider>
  );
};

// Hook personnalisé pour l'utiliser facilement dans vos composants
export const useToast = () => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast doit être utilisé à l'intérieur d'un ToastProvider");
  }
  return context;
};