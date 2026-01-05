import React from 'react';
// FIX: Dodali smo 'type' ovdje da kompajler zna da su ovo samo sučelja
import type { CloudDefinition, IslandDefinition } from '../types';

interface Props {
  definition: CloudDefinition | IslandDefinition;
  data: any[];
  type: 'cloud' | 'island';
}

export function DynamicTable({ definition, data, type }: Props) {
  const columns = React.useMemo(() => {
    if (type === 'cloud') {
      const def = definition as CloudDefinition;
      return def.fields.map(f => f.key);
    } else {
      const def = definition as IslandDefinition;
      return ['name', 'status', ...def.aggregations.map(a => a.name), 'updated_at'];
    }
  }, [definition, type]);

  if (!data || data.length === 0) {
    return <div className="p-4 text-slate-500 italic">No data found in {definition.name}.</div>;
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left text-sm">
        <thead className="bg-slate-800 text-slate-400 uppercase font-bold text-xs">
          <tr>
            {columns.map(col => (
              <th key={col} className="px-4 py-3">{col.replace(/_/g, ' ')}</th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-700">
          {data.map((row, i) => (
            <tr key={row.id || i} className="hover:bg-slate-800/50 transition-colors">
              {columns.map(col => (
                <td key={`${row.id}-${col}`} className="px-4 py-3">
                  {typeof row[col] === 'number' 
                    ? row[col].toLocaleString('hr-HR', { maximumFractionDigits: 2 }) 
                    : row[col] || '-'}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

