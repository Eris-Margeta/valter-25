import { useParams, Link } from "react-router-dom";
import { useEffect, useState } from "react";
import type { AppConfig } from "../types";
import { EntityForm } from "../components/EntityForm";
import { graphqlRequest } from "../api";
import { ArrowLeft } from "lucide-react";

interface EntityDetailProps {
  config: AppConfig | null;
  type: "cloud" | "island";
}

// ISPRAVAK: Definiramo tip za podatke
type EntityData = Record<string, unknown> | null;

export function EntityDetail({ config, type }: EntityDetailProps) {
  const { name, id } = useParams<{ name: string; id: string }>();
  const [data, setData] = useState<EntityData>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!name || !id) return;
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
        const all = (res.data[key] || []) as Record<string, unknown>[];
        const found = all.find((item) => 
            String(item.id) === id || String(item.name) === id
        );
        setData(found || null);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [name, id, type]);

  const handleSave = async (key: string, value: string) => {
     if (!name || !data) return false;
     
     const entityName = data.name as string;
     
     const mutation = `
       mutation($type: String!, $name: String!, $key: String!, $value: String!) {
         updateIslandField(islandType: $type, islandName: $name, key: $key, value: $value)
       }
     `;
     
     try {
       const res = await graphqlRequest(mutation, {
         type: name,
         name: entityName,
         key: key,
         value: value,
       });
       return res.data && res.data.updateIslandField === "Success";
     } catch (e) {
       console.error(e);
       return false;
     }
  };

  if (!config || !name) return <div>Loading...</div>;

  const definition =
    type === "cloud"
      ? config.CLOUDS.find((c) => c.name === name)
      : config.ISLANDS.find((i) => i.name === name);

  if (!definition) return <div>Entity Definition not found</div>;
  if (loading) return <div>Loading data...</div>;
  if (!data) return <div>Entity not found (ID: {id})</div>;

  return (
    <div className="p-8 max-w-4xl mx-auto space-y-8">
      <header>
        <Link to={`/list/${type}/${name}`} className="text-slate-400 hover:text-white flex items-center gap-1 mb-4">
            <ArrowLeft size={16} /> Back to List
        </Link>
        <h1 className="text-3xl font-bold text-white mb-2">{String(data.name) || id}</h1>
        <p className="text-slate-400">Editing {name}</p>
      </header>

      <EntityForm 
        definition={definition} 
        data={data} 
        onSave={handleSave} 
        readOnly={type === 'cloud'}
      />
      
      {type === 'cloud' && (
          <p className="text-yellow-500 text-sm mt-2">
            * Cloud entities are currently read-only in this version.
          </p>
      )}
    </div>
  );
}
