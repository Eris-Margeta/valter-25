import { useState, useEffect, useCallback } from 'react';
import { LayoutDashboard, Database, Folder, MessageSquare, Layers, RefreshCw, AlertCircle, RotateCcw } from 'lucide-react';
import { graphqlRequest, QUERIES, MUTATIONS } from './api';
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
  const [errorMsg, setErrorMsg] = useState<string | null>(null); 

  const init = async () => {
    try {
      setErrorMsg(null);
      const cfgData = await graphqlRequest(QUERIES.GET_CONFIG);
      setConfig(cfgData.config);
      refreshActions();
    } catch (e) { 
      console.error(e);
      setErrorMsg(String(e));
    }
  };

  const refreshActions = async () => {
    try {
      const actionsData = await graphqlRequest(QUERIES.GET_PENDING_ACTIONS);
      setPendingActions(actionsData.pendingActions);
    } catch (e) { console.error(e); }
  }

  const handleRescan = async () => {
    setLoading(true);
    try {
        await graphqlRequest(MUTATIONS.RESCAN_ISLANDS);
        // Wait a sec for backend to process
        setTimeout(async () => {
            await refreshActions();
            await fetchData();
            setLoading(false);
        }, 1000);
    } catch (e) {
        alert("Rescan failed: " + e);
        setLoading(false);
    }
  };

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
  }, [activeView, config]);

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

  if (errorMsg) {
    return (
      <div className="min-h-screen bg-slate-950 text-red-400 flex flex-col items-center justify-center font-mono p-8 text-center">
        <AlertCircle className="w-12 h-12 mb-4" />
        <h2 className="text-xl font-bold mb-2">CONNECTION FAILED</h2>
        <p className="text-sm text-slate-500 mb-6 max-w-md">{errorMsg}</p>
        <button onClick={() => window.location.reload()} className="bg-slate-800 hover:bg-slate-700 text-white px-4 py-2 rounded transition-colors">Retry Connection</button>
      </div>
    );
  }

  if (!config) return <div className="min-h-screen bg-slate-900 text-emerald-500 flex items-center justify-center font-mono animate-pulse">INITIALIZING VALTER LINK...</div>;

  return (
    <div className="min-h-screen bg-slate-900 text-slate-100 flex font-sans">
      <aside className="w-64 border-r border-slate-800 bg-slate-950 flex flex-col">
        <div className="p-6 border-b border-slate-800">
          <div className="flex items-center gap-2 text-emerald-500 mb-1">
            <Layers className="w-6 h-6" />
            <span className="font-bold tracking-widest text-lg">VALTER</span>
          </div>
          <div className="text-[10px] text-slate-500 font-mono uppercase">Open Source ERP v0.1</div>
        </div>
        <nav className="flex-1 p-4 space-y-6 overflow-y-auto">
          <div onClick={() => setActiveView({ type: 'dashboard' })} className={`flex items-center gap-3 px-3 py-2 rounded cursor-pointer transition-colors ${activeView.type === 'dashboard' ? 'bg-emerald-900/30 text-emerald-400 border border-emerald-900' : 'text-slate-400 hover:text-white hover:bg-slate-800'}`}><LayoutDashboard size={18} /><span className="text-sm font-medium">Command Center</span></div>
          
          <div>
            <div className="px-3 text-[10px] font-bold text-slate-600 uppercase tracking-wider mb-2">Clouds (SQL)</div>
            {config.CLOUDS.map(cloud => (
              <div key={cloud.name} onClick={() => setActiveView({ type: 'cloud', name: cloud.name })} className={`flex items-center gap-3 px-3 py-2 rounded cursor-pointer transition-colors ${activeView.name === cloud.name ? 'bg-slate-800 text-emerald-400' : 'text-slate-400 hover:text-white hover:bg-slate-800'}`}>
                <Database size={16} />
                <span className="text-sm">{cloud.name}</span>
              </div>
            ))}
          </div>

          <div>
            <div className="px-3 text-[10px] font-bold text-slate-600 uppercase tracking-wider mb-2">Islands (FS)</div>
            {config.ISLANDS.map(island => (
              <div key={island.name} onClick={() => setActiveView({ type: 'island', name: island.name })} className={`flex items-center gap-3 px-3 py-2 rounded cursor-pointer transition-colors ${activeView.name === island.name ? 'bg-slate-800 text-emerald-400' : 'text-slate-400 hover:text-white hover:bg-slate-800'}`}>
                <Folder size={16} />
                <span className="text-sm">{island.name}</span>
              </div>
            ))}
          </div>
        </nav>
      </aside>

      <main className="flex-1 flex flex-col h-screen overflow-hidden bg-slate-900">
        <header className="h-16 border-b border-slate-800 flex items-center justify-between px-8 bg-slate-900">
          <h1 className="text-xl font-semibold tracking-tight">{activeView.type === 'dashboard' ? 'Overview' : activeView.name}</h1>
          <div className="flex items-center gap-4">
            <button 
                onClick={handleRescan} 
                className="flex items-center gap-2 px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-xs text-slate-300 rounded border border-slate-700 transition-colors"
                title="Force Rescan of Filesystem"
            >
                <RotateCcw size={12} className={loading ? "animate-spin" : ""} />
                RESCAN SYSTEM
            </button>
            {loading && <RefreshCw className="w-4 h-4 animate-spin text-emerald-500" />}
            <div className="h-8 px-3 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center font-bold text-xs text-emerald-400">
              ORACLE ACTIVE
            </div>
          </div>
        </header>
        
        <div className="flex-1 overflow-y-auto p-8">
          {activeView.type === 'dashboard' ? (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <section className="bg-slate-800/30 p-6 rounded-xl border border-emerald-500/20 shadow-lg shadow-emerald-900/5">
                <h2 className="text-lg font-semibold mb-4 flex items-center gap-2 text-emerald-400"><MessageSquare className="w-5 h-5" />Oracle Interface</h2>
                <textarea 
                  className="w-full bg-slate-950 border border-slate-700 rounded p-4 text-sm focus:outline-none focus:border-emerald-500 h-32 resize-none mb-4 text-slate-300 placeholder-slate-600" 
                  placeholder="Ask questions about your data..." 
                  value={oracleQ} 
                  onChange={e => setOracleQ(e.target.value)} 
                />
                <button onClick={askOracle} disabled={loading} className="bg-emerald-600 hover:bg-emerald-500 px-6 py-2 rounded text-sm font-semibold transition-colors disabled:opacity-50 text-white shadow-lg">
                  Execute Query
                </button>
                {oracleA && <div className="mt-4 p-4 bg-slate-950 rounded border-l-2 border-emerald-500 text-sm leading-relaxed text-slate-300 animate-in fade-in slide-in-from-top-2">{oracleA}</div>}
              </section>
            </div>
          ) : (
            <div className="bg-slate-800/30 rounded-xl border border-slate-700 overflow-hidden shadow-xl">
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
