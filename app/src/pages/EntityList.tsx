import { useParams, useNavigate } from "react-router-dom";
import { useEffect, useState } from "react";
import type { AppConfig } from "../types";
import { DynamicTable } from "../components/DynamicTable";
import { graphqlRequest } from "../api";

interface EntityListProps {
  config: AppConfig | null;
  type: "cloud" | "island";
}

// ISPRAVAK: Definiramo tip za podatke
type EntityData = Record<string, unknown>;

export function EntityList({ config, type }: EntityListProps) {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [data, setData] = useState<EntityData[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!name) return;
    // ISPRAVAK: Isključujemo pravilo za ovu liniju
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLoading(true);
    const query =
      type === "cloud"
        ? `query { cloudData(name: "${name}") }`
        : `query { islandData(name: "${name}") }`;

    graphqlRequest(query)
      .then((res) => {
        const key = type === "cloud" ? "cloudData" : "islandData";
        setData(res.data[key] || []);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [name, type]);

  if (!config || !name) return <div className="p-8 text-white">Loading...</div>;

  const definition =
    type === "cloud"
      ? config.CLOUDS.find((c) => c.name === name)
      : config.ISLANDS.find((i) => i.name === name);

  if (!definition) return <div className="p-8 text-white">Entity not found</div>;

  return (
    <div className="p-8 max-w-7xl mx-auto space-y-8">
      <header>
        <h1 className="text-3xl font-bold text-white mb-2">{name} List</h1>
        <p className="text-slate-400">Manage all {name} entries</p>
      </header>

      {loading ? (
        <div className="text-slate-400">Loading data...</div>
      ) : (
        <DynamicTable
          data={data}
          title={name}
          onRowClick={(row) => {
            const id = row.id || row.name;
            if (id) {
               navigate(`/entity/${type}/${name}/${String(id)}`);
            }
          }}
        />
      )}
    </div>
  );
}
