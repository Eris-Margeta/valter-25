import React from "react";
import { ChevronRight } from "lucide-react";

interface DynamicTableProps {
  data: any[];
  title?: string;
  onRowClick?: (row: any) => void;
}

export function DynamicTable({ data, title, onRowClick }: DynamicTableProps) {
  if (!data || data.length === 0) {
    return (
      <div className="text-slate-500 italic p-4 border border-slate-800 rounded-lg">
        No data available for {title}
      </div>
    );
  }

  // Infer columns from the first row, excluding complex objects/arrays
  const columns = Object.keys(data[0]).filter((key) => {
    const val = data[0][key];
    return typeof val !== "object" || val === null;
  });

  return (
    <div className="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden shadow-sm">
      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm">
          <thead className="bg-slate-800/50 text-slate-400 uppercase font-bold text-xs">
            <tr>
              {columns.map((col) => (
                <th key={col} className="px-6 py-4 tracking-wider">
                  {col.replace(/_/g, " ")}
                </th>
              ))}
              <th className="px-6 py-4 w-10"></th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800">
            {data.map((row, i) => (
              <tr
                key={i}
                onClick={() => onRowClick && onRowClick(row)}
                className={`transition-colors group ${
                  onRowClick
                    ? "cursor-pointer hover:bg-slate-800 hover:text-white"
                    : ""
                }`}
              >
                {columns.map((col) => (
                  <td key={col} className="px-6 py-4 text-slate-300 group-hover:text-white transition-colors">
                    {String(row[col] ?? "-")}
                  </td>
                ))}
                <td className="px-6 py-4 text-slate-600 group-hover:text-indigo-400">
                  {onRowClick && <ChevronRight size={16} />}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="px-6 py-3 bg-slate-800/30 border-t border-slate-800 text-xs text-slate-500 text-right">
        Showing {data.length} entries
      </div>
    </div>
  );
}