import { AlertTriangle, GitMerge, ArrowRight } from 'lucide-react';
import type { PendingAction } from '../types';

interface Props {
  actions: PendingAction[];
  onResolve: (id: string, choice: 'APPROVE' | 'REJECT') => Promise<void>;
  onMerge: (action: PendingAction, suggestion: string) => Promise<void>;
}

export function ActionCenter({ actions, onResolve, onMerge }: Props) {
  if (actions.length === 0) return null;

  return (
    <div className="fixed bottom-6 right-6 w-96 bg-slate-800 border border-yellow-500/50 rounded-lg shadow-2xl overflow-hidden z-50 flex flex-col max-h-[80vh]">
      <div className="bg-yellow-500/10 p-3 border-b border-yellow-500/20 flex items-center gap-2 shrink-0">
        <AlertTriangle className="w-5 h-5 text-yellow-500 animate-pulse" />
        <h3 className="font-bold text-yellow-100 text-sm">Conflict Resolution ({actions.length})</h3>
      </div>
      <div className="overflow-y-auto p-4 space-y-4">
        {actions.map(action => (
          <div key={action.id} className="bg-slate-900 p-3 rounded border border-slate-700 shadow-sm">
            <p className="text-[10px] text-slate-500 uppercase tracking-wide mb-2">Unknown Entry</p>
            <div className="flex items-center gap-2 mb-4 bg-black/20 p-2 rounded">
               <span className="text-red-400 line-through decoration-red-500/50">{action.value}</span>
               <ArrowRight size={14} className="text-slate-500" />
               <span className="text-yellow-400 font-bold">?</span>
            </div>
            
            {/* Suggestions (Merge options) */}
            {action.suggestions && action.suggestions.length > 0 && (
               <div className="mb-4 space-y-2">
                 <p className="text-xs text-slate-400 flex items-center gap-1"><GitMerge size={12}/> Link to existing:</p>
                 {action.suggestions.map(s => (
                   <button 
                     key={s}
                     onClick={() => onMerge(action, s)}
                     className="w-full text-left text-xs bg-blue-900/20 hover:bg-blue-900/40 text-blue-300 border border-blue-800/30 p-2 rounded transition-colors flex items-center justify-between group"
                   >
                     {s}
                     <span className="opacity-0 group-hover:opacity-100 text-[10px] uppercase font-bold">Fix File</span>
                   </button>
                 ))}
               </div>
            )}

            <div className="flex gap-2 border-t border-slate-800 pt-3">
              <button 
                onClick={() => onResolve(action.id, 'APPROVE')}
                className="flex-1 bg-green-900/20 hover:bg-green-900/40 text-green-400 py-2 rounded text-xs font-semibold transition-colors"
              >
                Create New
              </button>
              <button 
                onClick={() => onResolve(action.id, 'REJECT')}
                className="flex-1 bg-slate-800 hover:bg-slate-700 text-slate-400 py-2 rounded text-xs font-semibold transition-colors"
              >
                Ignore
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}