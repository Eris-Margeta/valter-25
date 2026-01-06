import { Home, Database, Folder, Settings, Activity } from "lucide-react";
import { Link, useLocation } from "react-router-dom";
import { AppConfig } from "../types";

interface LayoutProps {
  children: React.ReactNode;
  config: AppConfig | null;
}

export function Layout({ children, config }: LayoutProps) {
  const location = useLocation();

  const isActive = (path: string) => location.pathname === path;
  const linkClass = (path: string) =>
    `flex items-center gap-3 px-3 py-2 rounded-md transition-colors ${
      isActive(path)
        ? "bg-slate-800 text-white"
        : "text-slate-400 hover:text-white hover:bg-slate-800/50"
    }`;

  return (
    <div className="flex h-screen bg-slate-950 text-slate-200 font-sans overflow-hidden">
      {/* Sidebar */}
      <aside className="w-64 bg-slate-900 border-r border-slate-800 flex flex-col">
        <div className="p-4 border-b border-slate-800 flex items-center gap-2">
          <div className="w-8 h-8 bg-indigo-500 rounded-lg flex items-center justify-center">
            <span className="font-bold text-white text-xl">V</span>
          </div>
          <span className="font-bold text-lg tracking-tight">VALTER</span>
        </div>

        <nav className="flex-1 p-4 space-y-6 overflow-y-auto">
          <div className="space-y-1">
            <Link to="/" className={linkClass("/")}>
              <Home size={18} />
              <span>Dashboard</span>
            </Link>
          </div>

          {config && (
            <>
              {config.CLOUDS.length > 0 && (
                <div className="space-y-1">
                  <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider px-3 mb-2">
                    Clouds
                  </h3>
                  {config.CLOUDS.map((cloud) => (
                    <Link
                      key={cloud.name}
                      to={`/list/cloud/${cloud.name}`}
                      className={linkClass(`/list/cloud/${cloud.name}`)}
                    >
                      <Database size={18} />
                      <span>{cloud.name}</span>
                    </Link>
                  ))}
                </div>
              )}

              {config.ISLANDS.length > 0 && (
                <div className="space-y-1">
                  <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider px-3 mb-2">
                    Islands
                  </h3>
                  {config.ISLANDS.map((island) => (
                    <Link
                      key={island.name}
                      to={`/list/island/${island.name}`}
                      className={linkClass(`/list/island/${island.name}`)}
                    >
                      <Folder size={18} />
                      <span>{island.name}</span>
                    </Link>
                  ))}
                </div>
              )}
            </>
          )}
        </nav>

        <div className="p-4 border-t border-slate-800">
           <Link to="/settings" className={linkClass("/settings")}>
            <Settings size={18} />
            <span>Settings</span>
          </Link>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto bg-slate-950 relative">
        {children}
      </main>
    </div>
  );
}
