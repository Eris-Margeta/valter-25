import React, { useState } from 'react';
import type { CloudDefinition, IslandDefinition } from '../types';
import { graphqlRequest, MUTATIONS } from '../api';
import { Check, X, Edit2 } from 'lucide-react';

interface Props {
  definition: CloudDefinition | IslandDefinition;
  data: any[];
  type: 'cloud' | 'island';
  onUpdate?: () => void; // Refresh callback
}

export function DynamicTable({ definition, data, type, onUpdate }: Props) {
  const [editingCell, setEditingCell] = useState<{ id: string, col: string, val: string } | null>(null);

  const columns = React.useMemo(() => {
    if (type === 'cloud') {
      const def = definition as CloudDefinition;
      return def.fields.map(f => f.key);
    } else {
      const def = definition as IslandDefinition;
      // Samo određena polja su editabilna (ona iz meta.yaml, ne agregacije)
      return ['name', 'status', ...def.aggregations.map(a => a.name), 'updated_at'];
    }
  }, [definition, type]);

  // Koja polja smijemo editirati? (Samo osnovna, ne izračunata)
  const isEditable = (col: string) => {
    if (type === 'cloud') return false; // Cloud editiranje još nije podržano u FsWriteru
    if (col === 'updated_at' || col.startsWith('ukupno') || col.startsWith('total')) return false;
    return true;
  };

  const handleSave = async () => {
    if (!editingCell) return;
    try {
      // Pretpostavljamo da red ima 'name' polje koje je ID projekta
      const row = data.find(d => d.id === editingCell.id);
      if (!row) return;

      await graphqlRequest(MUTATIONS.UPDATE_ISLAND_FIELD, {
        type: definition.name,
        name: row.name, // Koristimo ime projekta kao identifikator
        key: editingCell.col,
        value: editingCell.val
      });
      
      setEditingCell(null);
      if (onUpdate) onUpdate();
    } catch (e) {
      alert("Update failed: " + e);
    }
  };

  return (
    <div className="overflow-x-auto min-h-[300px]">
      <table className="w-full text-left text-sm">
        <thead className="bg-slate-800 text-slate-400 uppercase font-bold text-xs sticky top-0">
          <tr>
            {columns.map(col => (
              <th key={col} className="px-4 py-3">{col.replace(/_/g, ' ')}</th>
            ))}
            <th className="px-4 py-3 w-10"></th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-700">
          {data.map((row, i) => (
            <tr key={row.id || i} className="hover:bg-slate-800/50 transition-colors group">
              {columns.map(col => (
                <td key={`${row.id}-${col}`} className="px-4 py-3 relative">
                  {editingCell?.id === row.id && editingCell?.col === col ? (
                    <div className="flex items-center gap-1 absolute inset-0 px-2 bg-slate-800 z-10">
                      <input 
                        autoFocus
                        className="w-full bg-slate-900 border border-blue-500 rounded px-2 py-1 text-white outline-none"
                        value={editingCell.val}
                        onChange={e => setEditingCell({...editingCell, val: e.target.value})}
                        onKeyDown={e => {
                          if (e.key === 'Enter') handleSave();
                          if (e.key === 'Escape') setEditingCell(null);
                        }}
                      />
                      <button onClick={handleSave} className="text-green-400 hover:text-green-300"><Check size={14}/></button>
                      <button onClick={() => setEditingCell(null)} className="text-red-400 hover:text-red-300"><X size={14}/></button>
                    </div>
                  ) : (
                    <div 
                      className={`flex items-center gap-2 ${isEditable(col) ? 'cursor-pointer hover:text-blue-300' : ''}`}
                      onClick={() => {
                        if (isEditable(col)) setEditingCell({ id: row.id, col, val: String(row[col] || '') });
                      }}
                    >
                      <span>
                        {typeof row[col] === 'number' 
                          ? row[col].toLocaleString('hr-HR', { maximumFractionDigits: 2 }) 
                          : row[col] || '-'}
                      </span>
                      {isEditable(col) && <Edit2 size={10} className="opacity-0 group-hover:opacity-30" />}
                    </div>
                  )}
                </td>
              ))}
              <td className="px-4 py-3"></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

