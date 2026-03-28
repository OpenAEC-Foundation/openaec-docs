import { brand } from "../../config/brand";
import { useAuth } from "../../context/AuthContext";

export function AppBar() {
  const { user, login, logout } = useAuth();

  return (
    <div
      className="flex items-center justify-between h-10 px-3 text-sm select-none"
      style={{ backgroundColor: "var(--brand-header-bg)", color: "var(--brand-header-text)" }}
    >
      <div className="flex items-center gap-2">
        <span className="font-medium opacity-70">{brand.namePrefix}</span>
        <span className="font-bold" style={{ color: "var(--brand-primary)" }}>
          {brand.nameAccent}
        </span>
        <span className="opacity-50">|</span>
        <span className="font-medium">{brand.product}</span>
      </div>

      <div className="flex items-center gap-3">
        {user ? (
          <>
            <span className="opacity-70 text-xs">{user.name}</span>
            <button
              onClick={logout}
              className="opacity-50 hover:opacity-100 transition-opacity text-xs"
            >
              Uitloggen
            </button>
          </>
        ) : (
          <button
            onClick={login}
            className="opacity-70 hover:opacity-100 transition-opacity text-xs"
          >
            Inloggen
          </button>
        )}
      </div>
    </div>
  );
}
