import React, { useState, useEffect, useCallback } from 'react';
import { Terminal, LayoutDashboard, Database, Folder, MessageSquare, Menu, RefreshCw, Layers } from 'lucide-react';
import { graphqlRequest, QUERIES } from './api';
import type { AppConfig, PendingAction } from './types';
import { DynamicTable } from './components/DynamicTable';
import { ActionCenter } from './components/ActionCenter';

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [activeView, setActiveView] = useState<{ type: 'dashboard' | 'cloud' | 'island' | 'pending_actions', name?: string }>({ type: 'dashboard' });
  const [tableData, setTableData] = useState<any[]>([]);
  const [pendingActions, setPendingActions] = useState<PendingAction[]>([]);
  const [oracleQ, setOracleQ] = useState('');
  const [oracleA, setOracleA] = useState('');
  const [loading, setLoading] = useState(false);

  const init = async () => {
    try {
      const cfgData = await graphqlRequest(QUERIES.GET_CONFIG);
      setConfig(cfgData.config);
      refreshActions();
    } catch (e) { console.error(e); }
  };

  const refreshActions = async () => {
    try {
      const actionsData = await graphqlRequest(QUERIES.GET_PENDING_ACTIONS);
      setPendingActions(actionsData.pendingActions);
    } catch (e) { console.error(e); }
  }

  // Izdvojeno u useCallback da možemo zvati iz child komponenti
  const fetchData = useCallback(async () => {
    if (!config) return;
    setLoading(true);
    try {
      let data;
      if (activeView.type === 'cloud') {
        data = await graphqlRequest(QUERIES.GET_CLOUD_DATA, { name: activeView.name! });
        setTableData(data.cloudData);
      } else if (activeView.type === 'island') {
        data = await graphqlRequest(QUERIES.GET_ISLAND_DATA, { name: activeView.name! });
        setTableData(data.islandData);
      } else if (activeView.type === 'pending_actions') {
        await refreshActions();
        setTableData(pendingActions);
      }
    } catch (e) { console.error(e); }
    finally { setLoading(false); }
  }, [activeView, config, pendingActions]); // dependencies

  useEffect(() => { init(); }, []);
  useEffect(() => { fetchData(); }, [fetchData]);

  const askOracle = async () => {
    if (!oracleQ) return;
    setLoading(true);
    try {
      const res = await graphqlRequest(QUERIES.ASK_ORACLE, { q: oracleQ });
      setOracleA(res.askOracle);
    } catch (e) { setOracleA("Oracle disconnected."); }
    setLoading(false);
  };

  if (!config) return <div className="min-h-screen bg-slate-900 text-white flex items-center justify-center">Loading Strata Engine...</div>;

  return (
    <div className="min-h-screen bg-slate-900 text-slate-100 flex">
      <aside className="w-64 border-r border-slate-800 bg-slate-900/50 flex flex-col">
        <div className="p-6 border-b border-slate-800">
          <div className="flex items-center gap-2 text-blue-500 mb-1">
            <Layers className="w-6 h-6" />
            <span className="font-bold tracking-widest">STRATA</span>
          </div>
          <div className="text-xs text-slate-500 font-mono uppercase">{config.GLOBAL.company_name}</div>
        </div>
        <nav className="flex-1 p-4 space-y-6 overflow-y-auto">
          <div onClick={() => setActiveView({ type: 'dashboard' })} className={`flex items-center gap-3 px-3 py-2 rounded cursor-pointer transition-colors ${activeView.type === 'dashboard' ? 'bg-blue-600 text-white' : 'text-slate-400 hover:text-white hover:bg-slate-800'}`}><LayoutDashboard size={18} /><span className="text-sm font-medium">Dashboard</span></div>
          <div><div className="px-3 text-[10px] font-bold text-slate-600 uppercase tracking-wider mb-2">Clouds</div>{config.CLOUDS.map(cloud => (<div key={cloud.name} onClick={() => setActiveView({ type: 'cloud', name: cloud.name })} className={`flex items-center gap-3 px-3 py-2 rounded cursor-pointer transition-colors ${activeView.name === cloud.name ? 'bg-slate-800 text-blue-400' : 'text-slate-400 hover:text-white hover:bg-slate-800'}`}><Database size={16} /><span className="text-sm">{cloud.name}</span></div>))}</div>
          <div><div className="px-3 text-[10px] font-bold text-slate-600 uppercase tracking-wider mb-2">Islands</div>{config.ISLANDS.map(island => (<div key={island.name} onClick={() => setActiveView({ type: 'island', name: island.name })} className={`flex items-center gap-3 px-3 py-2 rounded cursor-pointer transition-colors ${activeView.name === island.name ? 'bg-slate-800 text-blue-400' : 'text-slate-400 hover:text-white hover:bg-slate-800'}`}><Folder size={16} /><span className="text-sm">{island.name}</span></div>))}</div>
        </nav>
      </aside>
      <main className="flex-1 flex flex-col h-screen overflow-hidden">
        <header className="h-16 border-b border-slate-800 flex items-center justify-between px-8"><h1 className="text-xl font-semibold">{activeView.type === 'dashboard' ? 'Overview' : activeView.name}</h1><div className="flex items-center gap-4">{loading && <RefreshCw className="w-4 h-4 animate-spin text-blue-500" />}<div className="w-8 h-8 rounded-full bg-blue-600 flex items-center justify-center font-bold text-xs">AI</div></div></header>
        <div className="flex-1 overflow-y-auto p-8">
          {activeView.type === 'dashboard' ? (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <section className="bg-slate-800/50 p-6 rounded-xl border border-blue-900/30">
                <h2 className="text-lg font-semibold mb-4 flex items-center gap-2"><MessageSquare className="w-5 h-5 text-blue-400" />Oracle Interface</h2>
                <textarea className="w-full bg-slate-900 border border-slate-700 rounded p-4 text-sm focus:outline-none focus:border-blue-500 h-32 resize-none mb-4" placeholder={`Ask questions about ${config.GLOBAL.company_name} data...`} value={oracleQ} onChange={e => setOracleQ(e.target.value)} />
                <button onClick={askOracle} disabled={loading} className="bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded text-sm font-semibold transition-colors disabled:opacity-50">Consult Oracle</button>
                {oracleA && <div className="mt-4 p-4 bg-slate-900 rounded border-l-2 border-blue-500 text-sm leading-relaxed text-slate-300">{oracleA}</div>}
              </section>
              <section className="bg-slate-800/50 p-6 rounded-xl border border-slate-700">
                <h2 className="text-lg font-semibold mb-4">System Status</h2>
                <div className="grid grid-cols-2 gap-4">
                   <div className="bg-slate-900 p-4 rounded text-center"><div className="text-2xl font-bold text-green-400">Online</div><div className="text-xs text-slate-500 uppercase mt-1">Daemon Status</div></div>
                   <div className="bg-slate-900 p-4 rounded text-center"><div className="text-2xl font-bold text-blue-400">{config.CLOUDS.length + config.ISLANDS.length}</div><div className="text-xs text-slate-500 uppercase mt-1">Active Definitions</div></div>
                </div>
              </section>
            </div>
          ) : (
            <div className="bg-slate-800/50 rounded-xl border border-slate-700 overflow-hidden">
              {activeView.name && config.CLOUDS.find(c => c.name === activeView.name) && (<DynamicTable type="cloud" definition={config.CLOUDS.find(c => c.name === activeView.name)!} data={tableData} onUpdate={fetchData} />)}
              {activeView.name && config.ISLANDS.find(i => i.name === activeView.name) && (<DynamicTable type="island" definition={config.ISLANDS.find(i => i.name === activeView.name)!} data={tableData} onUpdate={fetchData} />)}
            </div>
          )}
        </div>
      </main>
      <ActionCenter actions={pendingActions} onResolve={() => { refreshActions(); fetchData(); }} />
    </div>
  );
}
export default App;

