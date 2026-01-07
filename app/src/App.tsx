import { useEffect, useState } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Layout } from "./components/Layout";
import { DashboardHome } from "./pages/DashboardHome";
import { EntityList } from "./pages/EntityList";
import { EntityDetail } from "./pages/EntityDetail";
import type { AppConfig, PendingAction, ConfigStatus } from "./types";
import { graphqlRequest, MUTATIONS } from "./api";
import { listen } from "@tauri-apps/api/event";
import { AlertTriangle, RefreshCw, LoaderCircle } from "lucide-react";

function ConfigErrorOverlay({ status, onRecheck }: { status: ConfigStatus, onRecheck: () => void }) {
  if (status.type !== 'runtimeError') {
    return null;
  }

  return (
    <div className="fixed inset-0 bg-slate-950/90 z-[100] flex items-center justify-center p-8 backdrop-blur-sm">
      <div className="bg-slate-900 border border-red-500/50 rounded-lg max-w-2xl w-full p-8 shadow-2xl text-center">
        <AlertTriangle className="w-16 h-16 text-red-500 mx-auto mb-6 animate-pulse" />
        <h2 className="text-2xl font-bold text-white mb-3">Greška u Konfiguraciji</h2>
        <p className="text-slate-400 mb-6">
          VALTER je konfiguriran da koristi sistemske varijable okruženja, ali neke od njih nedostaju ili su prazne.
          Da biste nastavili, molimo vas da definirate sve potrebne varijable u vašem sistemskom okruženju.
        </p>
        <div className="bg-slate-800/50 p-4 rounded-md text-left mb-6">
          <h3 className="text-sm font-semibold text-slate-300 mb-2">Nedostajuće Varijable:</h3>
          <ul className="list-disc list-inside text-red-400 font-mono text-xs space-y-1">
            {status.missingKeys.map(key => <li key={key}>{key}</li>)}
          </ul>
        </div>
        <p className="text-xs text-slate-500 mb-6">
          Alternativno, ako želite privremeno ignorirati sistemske varijable i koristiti one ugrađene pri kompajliranju,
          postavite sistemsku varijablu `VALTER_IGNORE_ENV_DURING_RUNTIME=true`.
        </p>
        <button
          onClick={onRecheck}
          className="bg-indigo-600 hover:bg-indigo-500 text-white font-bold py-3 px-6 rounded-lg transition-colors flex items-center gap-2 mx-auto"
        >
          <RefreshCw size={16} />
          Ponovno Provjeri Konfiguraciju
        </button>
      </div>
    </div>
  );
}


function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [pendingActions, setPendingActions] = useState<PendingAction[]>([]);
  const [configStatus, setConfigStatus] = useState<ConfigStatus | null>(null);

  const fetchData = async () => {
    try {
      // Koristimo graphqlRequest koji sada vraća cijeli objekt, uključujući `data`
      const response = await graphqlRequest(`
        query {
          config
          pendingActions
          envConfigStatus
        }
      `);
      
      if (response.data) {
        setConfig(response.data.config);
        setPendingActions(response.data.pendingActions);
        setConfigStatus(response.data.envConfigStatus);
      } else {
        // Ako `data` ne postoji, vjerojatno je GraphQL vratio grešku
        console.error("GraphQL response missing data:", response.errors);
      }

    } catch (e) {
      console.error("Failed to fetch data", e);
      // Ovdje možemo postaviti neki globalni error state ako API nije dostupan
    }
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 5000);

    const unlistenPromise = listen("menu-rescan", () => {
      console.log("Native Rescan Event Received");
      handleRescan();
    });

    return () => {
      clearInterval(interval);
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const handleResolveAction = async (id: string, choice: "APPROVE" | "REJECT") => {
    await graphqlRequest(MUTATIONS.RESOLVE_ACTION, { actionId: id, choice });
    fetchData();
  };
  
  const handleMergeAction = async (action: PendingAction, suggestion: string) => {
    try {
      const ctx = JSON.parse(action.context);
      await graphqlRequest(MUTATIONS.UPDATE_ISLAND_FIELD, {
        islandType: ctx.source_island_type,
        islandName: ctx.source_island_name, 
        key: ctx.field,
        value: suggestion 
      });
      await graphqlRequest(MUTATIONS.RESOLVE_ACTION, { actionId: action.id, choice: 'REJECT' });
      fetchData();
    } catch (e) {
      alert("Auto-fix failed: " + e);
    }
  };

  const handleRescan = async () => {
    try {
      await graphqlRequest(MUTATIONS.RESCAN_ISLANDS);
      alert("Rescan started.");
      fetchData();
    } catch (e) {
      console.error("Rescan failed", e);
    }
  };

  // ====================================================================
  // NOVI, ISPRAVLJENI RENDER BLOK
  // ====================================================================

  // 1. Ako imamo blokirajuću grešku, prikaži SAMO nju.
  if (configStatus?.type === 'runtimeError') {
    return <ConfigErrorOverlay status={configStatus} onRecheck={fetchData} />;
  }

  // 2. Ako još učitavamo osnovnu konfiguraciju, prikaži globalni loader.
  if (!config) {
    return (
      <div className="flex h-screen w-full items-center justify-center bg-slate-950 text-slate-400">
        <LoaderCircle className="animate-spin mr-2" />
        Inicijalizacija VALTER sustava...
      </div>
    );
  }

  // 3. Ako je sve u redu, prikaži cijelu aplikaciju.
  return (
    <BrowserRouter>
      <Layout config={config}>
        <Routes>
          <Route
            path="/"
            element={
              <DashboardHome
                config={config}
                pendingActions={pendingActions}
                onResolveAction={handleResolveAction}
                onMergeAction={handleMergeAction}
                onRescan={handleRescan}
              />
            }
          />
          <Route path="/list/cloud/:name" element={<EntityList config={config} type="cloud" />} />
          <Route path="/list/island/:name" element={<EntityList config={config} type="island" />} />
          <Route path="/entity/cloud/:name/:id" element={<EntityDetail config={config} type="cloud" />} />
          <Route path="/entity/island/:name/:id" element={<EntityDetail config={config} type="island" />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}

export default App;
