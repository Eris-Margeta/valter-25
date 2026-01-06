import { useEffect, useState } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Layout } from "./components/Layout";
import { DashboardHome } from "./pages/DashboardHome";
import { EntityList } from "./pages/EntityList";
import { EntityDetail } from "./pages/EntityDetail";
import type { AppConfig, PendingAction } from "./types";
import { graphqlRequest, MUTATIONS } from "./api";
import { listen } from "@tauri-apps/api/event";

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [pendingActions, setPendingActions] = useState<PendingAction[]>([]);

  const fetchData = async () => {
    try {
      const configRes = await graphqlRequest("{ config }");
      setConfig(configRes.data.config);

      const actionsRes = await graphqlRequest("{ pendingActions }");
      setPendingActions(actionsRes.data.pendingActions);
    } catch (e) {
      console.error("Failed to fetch data", e);
    }
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 2000);

    // Listen for Native Menu Events
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
    await graphqlRequest(MUTATIONS.RESOLVE_ACTION, { id, choice });
    fetchData();
  };
  
  const handleMergeAction = async (action: PendingAction, suggestion: string) => {
    try {
      const ctx = JSON.parse(action.context);
      
      await graphqlRequest(MUTATIONS.UPDATE_ISLAND_FIELD, {
        type: ctx.source_island_type,
        name: ctx.source_island_name, 
        key: ctx.field,
        value: suggestion 
      });

      await graphqlRequest(MUTATIONS.RESOLVE_ACTION, { id: action.id, choice: 'REJECT' });
      
      fetchData();
    } catch (e) {
      alert("Auto-fix failed: " + e);
    }
  };

  const handleRescan = async () => {
    try {
      await graphqlRequest(MUTATIONS.RESCAN_ISLANDS);
      alert("Rescan started.");
    } catch (e) {
      console.error("Rescan failed", e);
    }
  };

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