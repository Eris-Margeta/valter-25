import { useState, useEffect } from "react";
import type { CloudDefinition, IslandDefinition } from "../types";
import { Save, AlertCircle, Check } from "lucide-react";

interface EntityFormProps {
  definition: CloudDefinition | IslandDefinition;
  data: Record<string, any>;
  onSave: (key: string, value: string) => Promise<boolean>;
  readOnly?: boolean;
}

export function EntityForm({ definition, data, onSave, readOnly = false }: EntityFormProps) {
  const [formData, setFormData] = useState(data);
  const [status, setStatus] = useState<Record<string, "idle" | "saving" | "success" | "error">>({});

  useEffect(() => {
    setFormData(data);
  }, [data]);

  const handleChange = async (key: string, value: string) => {
    setFormData((prev) => ({ ...prev, [key]: value }));
  };

  const handleBlur = async (key: string, value: string) => {
    if (readOnly) return;
    if (value === data[key]) return; // No change

    setStatus((prev) => ({ ...prev, [key]: "saving" }));
    const success = await onSave(key, value);
    setStatus((prev) => ({ ...prev, [key]: success ? "success" : "error" }));

    if (success) {
      setTimeout(() => {
        setStatus((prev) => ({ ...prev, [key]: "idle" }));
      }, 2000);
    }
  };

  const renderField = (key: string, value: any) => {
    const isEditing = status[key] === "saving";
    const isSuccess = status[key] === "success";
    const isError = status[key] === "error";

    return (
      <div key={key} className="flex flex-col gap-1 mb-4">
        <label className="text-xs uppercase font-semibold text-slate-500 tracking-wider">
          {key}
        </label>
        <div className="relative">
          <input
            type="text"
            className={`w-full bg-slate-900 border ${
              isError ? "border-red-500" : isSuccess ? "border-green-500" : "border-slate-700"
            } rounded px-3 py-2 text-white focus:outline-none focus:border-indigo-500 transition-colors`}
            value={value || ""}
            onChange={(e) => handleChange(key, e.target.value)}
            onBlur={(e) => handleBlur(key, e.target.value)}
            disabled={readOnly}
          />
          <div className="absolute right-3 top-2.5 text-slate-400">
             {isEditing && <Save size={16} className="animate-pulse" />}
             {isSuccess && <Check size={16} className="text-green-500" />}
             {isError && <AlertCircle size={16} className="text-red-500" />}
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
      <h2 className="text-xl font-bold text-white mb-6 flex items-center gap-2">
        {definition.name} Details
      </h2>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Render explicitly defined fields for Clouds */}
        {'fields' in definition && definition.fields.map((f) => 
           renderField(f.key, formData[f.key])
        )}

        {/* Render all data keys for Islands (since they lack explicit field config) */}
        {!('fields' in definition) && Object.keys(formData).map((key) => {
             if (typeof formData[key] === 'object') return null; // Skip nested objects for now
             return renderField(key, formData[key]);
        })}
      </div>
    </div>
  );
}
