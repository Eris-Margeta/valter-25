import type { AppConfig, PendingAction } from "../types";
import { ActionCenter } from "../components/ActionCenter";
import { Activity, Database, Folder } from "lucide-react";

interface DashboardHomeProps {
  config: AppConfig | null;
  pendingActions: PendingAction[];
  onResolveAction: (id: string, choice: "APPROVE" | "REJECT") => Promise<void>;
  onMergeAction: (action: PendingAction, suggestion: string) => Promise<void>;
  onRescan: () => Promise<void>;
}

export function DashboardHome({
  config,
  pendingActions,
  onResolveAction,
  onMergeAction,
  onRescan,
}: DashboardHomeProps) {
  if (!config) return <div className="p-8 text-white">Loading config...</div>;

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8">
      <header className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Dashboard</h1>
          <p className="text-slate-400">
            Overview of {config.GLOBAL.company_name}
          </p>
        </div>
        <button
          onClick={onRescan}
          className="bg-slate-800 hover:bg-slate-700 text-white px-4 py-2 rounded-md border border-slate-700 transition-colors flex items-center gap-2"
        >
          <Activity size={16} /> Rescan System
        </button>
      </header>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl flex items-center gap-4">
          <div className="p-3 bg-indigo-500/10 text-indigo-400 rounded-lg">
            <Database size={24} />
          </div>
          <div>
            <p className="text-slate-400 text-sm font-medium uppercase">Clouds</p>
            <p className="text-2xl font-bold text-white">
              {config.CLOUDS.length}
            </p>
          </div>
        </div>
        <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl flex items-center gap-4">
          <div className="p-3 bg-emerald-500/10 text-emerald-400 rounded-lg">
             <Folder size={24} />
          </div>
          <div>
            <p className="text-slate-400 text-sm font-medium uppercase">Islands</p>
            <p className="text-2xl font-bold text-white">
              {config.ISLANDS.length}
            </p>
          </div>
        </div>
        <div className="bg-slate-900 border border-slate-800 p-6 rounded-xl flex items-center gap-4">
          <div className="p-3 bg-amber-500/10 text-amber-400 rounded-lg">
            <Activity size={24} />
          </div>
          <div>
            <p className="text-slate-400 text-sm font-medium uppercase">
              Pending Actions
            </p>
            <p className="text-2xl font-bold text-white">
              {pendingActions.length}
            </p>
          </div>
        </div>
      </div>

      <ActionCenter 
        actions={pendingActions} 
        onResolve={onResolveAction} 
        onMerge={onMergeAction}
      />
    </div>
  );
}