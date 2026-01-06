import { useEffect, useState } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Layout } from "./components/Layout";
import { DashboardHome } from "./pages/DashboardHome";
import { EntityList } from "./pages/EntityList";
import { EntityDetail } from "./pages/EntityDetail";
import { AppConfig, PendingAction } from "./types";
import { graphqlRequest, MUTATIONS } from "./api";

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
    return () => clearInterval(interval);
  }, []);

  const handleResolveAction = async (id: string, choice: "APPROVE" | "REJECT") => {
    await graphqlRequest(MUTATIONS.RESOLVE_ACTION, { id, choice });
    fetchData();
  };
  
  const handleMergeAction = async (action: PendingAction, suggestion: string) => {
    try {
      // 1. Parse context to find the source error
      const ctx = JSON.parse(action.context);
      
      // 2. Call mutation to fix the file
      await graphqlRequest(MUTATIONS.UPDATE_ISLAND_FIELD, {
        type: ctx.source_island_type,
        name: ctx.source_island_name, 
        key: ctx.field,
        value: suggestion 
      });

      // 3. Reject the pending action as it is now resolved via fix
      await graphqlRequest(MUTATIONS.RESOLVE_ACTION, { id: action.id, choice: 'REJECT' });
      
      fetchData();
    } catch (e) {
      alert("Auto-fix failed: " + e);
    }
  };

  const handleRescan = async () => {
    await graphqlRequest(MUTATIONS.RESCAN_ISLANDS);
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
          
          {/* Lists */}
          <Route path="/list/cloud/:name" element={<EntityList config={config} type="cloud" />} />
          <Route path="/list/island/:name" element={<EntityList config={config} type="island" />} />
          
          {/* Details */}
          <Route path="/entity/cloud/:name/:id" element={<EntityDetail config={config} type="cloud" />} />
          <Route path="/entity/island/:name/:id" element={<EntityDetail config={config} type="island" />} />
          
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}

export default App;
