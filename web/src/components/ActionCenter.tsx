
import React from 'react';
import { AlertTriangle, Check, X, GitMerge } from 'lucide-react';
import { PendingAction } from '../types';
import { graphqlRequest, MUTATIONS } from '../api';

interface Props {
  actions: PendingAction[];
  onResolve: () => void; // Callback to refresh data
}

export function ActionCenter({ actions, onResolve }: Props) {
  if (actions.length === 0) return null;

  const handleResolve = async (id: string, choice: 'APPROVE' | 'REJECT') => {
    try {
      await graphqlRequest(MUTATIONS.RESOLVE_ACTION, { id, choice });
      onResolve();
    } catch (e) {
      alert("Error resolving action: " + e);
    }
  };

  return (
    <div className="fixed bottom-6 right-6 w-96 bg-slate-800 border border-yellow-500/50 rounded-lg shadow-2xl overflow-hidden z-50">
      <div className="bg-yellow-500/10 p-3 border-b border-yellow-500/20 flex items-center gap-2">
        <AlertTriangle className="w-5 h-5 text-yellow-500 animate-pulse" />
        <h3 className="font-bold text-yellow-100 text-sm">Action Required ({actions.length})</h3>
      </div>
      <div className="max-h-[60vh] overflow-y-auto p-4 space-y-4">
        {actions.map(action => (
          <div key={action.id} className="bg-slate-900 p-3 rounded border border-slate-700">
            <p className="text-xs text-slate-400 mb-1">{action.context}</p>
            <p className="text-sm font-medium text-white mb-2">
              Unknown {action.target_table}: <span className="text-yellow-400">"{action.value}"</span>
            </p>
            
            {/* Prijedlozi (Fuzzy Match) */}
            {action.suggestions && action.suggestions.length > 0 && (
               <div className="mb-3 bg-slate-800 p-2 rounded text-xs">
                 <p className="text-slate-500 mb-1 flex items-center gap-1"><GitMerge size={12}/> Did you mean?</p>
                 {action.suggestions.map(s => (
                   <div key={s} className="text-blue-300 font-mono ml-2">• {s}</div>
                 ))}
                 <div className="text-[10px] text-slate-500 mt-1 italic">
                   (Tip: Fix typo in file to auto-resolve)
                 </div>
               </div>
            )}

            <div className="flex gap-2 mt-2">
              <button 
                onClick={() => handleResolve(action.id, 'APPROVE')}
                className="flex-1 bg-green-900/30 hover:bg-green-900/50 text-green-400 border border-green-700/50 py-1 px-2 rounded text-xs flex items-center justify-center gap-1 transition-colors"
              >
                <Check size={14} /> Create New
              </button>
              <button 
                onClick={() => handleResolve(action.id, 'REJECT')}
                className="flex-1 bg-red-900/30 hover:bg-red-900/50 text-red-400 border border-red-700/50 py-1 px-2 rounded text-xs flex items-center justify-center gap-1 transition-colors"
              >
                <X size={14} /> Ignore
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
