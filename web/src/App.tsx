import React, { useState, useEffect } from 'react';
import { Database, User, PlusCircle, MessageSquare, Terminal, Search, AlertCircle, RefreshCw } from 'lucide-react';

const API_URL = 'http://localhost:8000/graphql';

interface Entity {
  id: string;
  name: string;
}

function App() {
  const [clients, setClients] = useState<Entity[]>([]);
  const [operators, setOperators] = useState<Entity[]>([]);
  const [oracleQuestion, setOracleQuestion] = useState('');
  const [oracleAnswer, setOracleAnswer] = useState('');
  const [isAsking, setIsAsking] = useState(false);
  const [newProject, setNewProject] = useState({ name: '', client: '', operator: '' });
  const [msg, setMsg] = useState('');
  const [status, setStatus] = useState<'online' | 'offline' | 'connecting'>('connecting');

  const fetchData = async () => {
    try {
      const res = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: '{ clients { id name } operators { id name } }'
        })
      });
      const data = await res.json();
      if (data.data) {
        setClients(data.data.clients);
        setOperators(data.data.operators);
        setStatus('online');
      } else {
        setStatus('offline');
      }
    } catch (err) {
      console.error('Fetch error:', err);
      setStatus('offline');
    }
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  }, []);

  const askOracle = async () => {
    if (!oracleQuestion) return;
    setIsAsking(true);
    try {
      const res = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: `query { askOracle(question: "${oracleQuestion}") }`
        })
      });
      const data = await res.json();
      setOracleAnswer(data.data.askOracle);
    } catch (err) {
      setOracleAnswer('The Oracle is currently silent (Connection Error).');
    } finally {
      setIsAsking(false);
    }
  };

  const createProject = async () => {
    try {
      const res = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: `mutation { createProject(name: "${newProject.name}", client: "${newProject.client}", operator: "${newProject.operator}") }`
        })
      });
      const data = await res.json();
      setMsg(data.data.createProject);
      setNewProject({ name: '', client: '', operator: '' });
      fetchData();
    } catch (err) {
      setMsg('Failed to create project (Connection Error).');
    }
  };

  return (
    <div className="min-h-screen bg-slate-900 text-slate-100 p-8">
      {/* Header */}
      <header className="max-w-6xl mx-auto flex items-center justify-between mb-12">
        <div className="flex items-center gap-3">
          <Terminal className="text-blue-500 w-8 h-8" />
          <h1 className="text-2xl font-bold tracking-tight">STRATA <span className="text-blue-500">DAEMON</span></h1>
        </div>
        <div className={`text-xs font-mono flex items-center gap-2 px-3 py-1 rounded-full border ${
          status === 'online' ? 'border-green-500/30 bg-green-500/10 text-green-400' : 
          status === 'connecting' ? 'border-yellow-500/30 bg-yellow-500/10 text-yellow-400' :
          'border-red-500/30 bg-red-500/10 text-red-400'
        }`}>
          <div className={`w-2 h-2 rounded-full ${
            status === 'online' ? 'bg-green-500 animate-pulse' : 
            status === 'connecting' ? 'bg-yellow-500' :
            'bg-red-500'
          }`} />
          SYSTEM STATUS: {status.toUpperCase()}
        </div>
      </header>

      {status === 'offline' && (
        <div className="max-w-6xl mx-auto mb-8 bg-red-900/20 border border-red-500/50 rounded-lg p-4 flex items-center gap-3 text-red-200">
          <AlertCircle className="w-6 h-6 shrink-0" />
          <div>
            <p className="font-semibold">Connection Lost</p>
            <p className="text-sm opacity-80">The Strata Daemon is unreachable. Ensure the backend is running via <code className="bg-black/30 px-1 rounded">cargo run</code>.</p>
          </div>
          <button onClick={fetchData} className="ml-auto hover:bg-red-500/20 p-2 rounded-full transition-colors">
            <RefreshCw className="w-5 h-5" />
          </button>
        </div>
      )}

      <main className={`max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-3 gap-8 transition-opacity duration-500 ${status === 'offline' ? 'opacity-50 pointer-events-none' : 'opacity-100'}`}>
        
        {/* Left Column: Stats & Creation */}
        <div className="space-y-8">
          <section className="bg-slate-800/50 p-6 rounded-xl border border-slate-700">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <PlusCircle className="w-5 h-5 text-blue-400" />
              Manifest New Project
            </h2>
            <div className="space-y-4">
              <input 
                className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-sm focus:outline-none focus:border-blue-500"
                placeholder="Project Name"
                value={newProject.name}
                onChange={e => setNewProject({...newProject, name: e.target.value})}
              />
              <input 
                className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-sm focus:outline-none focus:border-blue-500"
                placeholder="Client Name"
                value={newProject.client}
                onChange={e => setNewProject({...newProject, client: e.target.value})}
              />
              <input 
                className="w-full bg-slate-900 border border-slate-700 rounded p-2 text-sm focus:outline-none focus:border-blue-500"
                placeholder="Operator Name"
                value={newProject.operator}
                onChange={e => setNewProject({...newProject, operator: e.target.value})}
              />
              <button 
                onClick={createProject}
                className="w-full bg-blue-600 hover:bg-blue-500 py-2 rounded text-sm font-semibold transition-colors"
              >
                Create Island
              </button>
              {msg && <p className="text-xs text-blue-300 mt-2">{msg}</p>}
            </div>
          </section>

          <div className="grid grid-cols-2 gap-4">
             <div className="bg-slate-800/50 p-4 rounded-xl border border-slate-700 text-center">
                <div className="text-2xl font-bold text-blue-400">{clients.length}</div>
                <div className="text-xs text-slate-400 uppercase tracking-widest mt-1">Clients</div>
             </div>
             <div className="bg-slate-800/50 p-4 rounded-xl border border-slate-700 text-center">
                <div className="text-2xl font-bold text-blue-400">{operators.length}</div>
                <div className="text-xs text-slate-400 uppercase tracking-widest mt-1">Operators</div>
             </div>
          </div>
        </div>

        {/* Center: Lists */}
        <div className="lg:col-span-2 space-y-8">
          <section className="bg-slate-800/50 p-6 rounded-xl border border-slate-700 h-[300px] overflow-y-auto">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2 sticky top-0 bg-slate-800 py-1">
              <Database className="w-5 h-5 text-blue-400" />
              The Cloud: Clients
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {clients.map(c => (
                <div key={c.id} className="bg-slate-900/50 p-3 rounded border border-slate-700/50 flex items-center justify-between">
                  <span className="text-sm font-medium">{c.name}</span>
                  <span className="text-[10px] text-slate-500 font-mono">{c.id.slice(0,8)}</span>
                </div>
              ))}
            </div>
          </section>

          <section className="bg-slate-800/50 p-6 rounded-xl border border-slate-700 h-[300px] overflow-y-auto">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2 sticky top-0 bg-slate-800 py-1">
              <User className="w-5 h-5 text-blue-400" />
              The Cloud: Operators
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {operators.map(o => (
                <div key={o.id} className="bg-slate-900/50 p-3 rounded border border-slate-700/50 flex items-center justify-between">
                  <span className="text-sm font-medium">{o.name}</span>
                  <span className="text-[10px] text-slate-500 font-mono">{o.id.slice(0,8)}</span>
                </div>
              ))}
            </div>
          </section>

          {/* AI Interface */}
          <section className="bg-slate-800/50 p-6 rounded-xl border border-blue-900/50 relative overflow-hidden">
            <div className="absolute top-0 right-0 p-4 opacity-10">
              <MessageSquare className="w-24 h-24" />
            </div>
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <MessageSquare className="w-5 h-5 text-blue-400" />
              The Oracle
            </h2>
            <div className="relative">
              <textarea 
                className="w-full bg-slate-900 border border-slate-700 rounded p-4 text-sm focus:outline-none focus:border-blue-500 h-32 resize-none"
                placeholder="Ask the Oracle about your empire..."
                value={oracleQuestion}
                onChange={e => setOracleQuestion(e.target.value)}
              />
              <button 
                onClick={askOracle}
                disabled={isAsking}
                className="absolute bottom-4 right-4 bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded text-xs font-bold uppercase transition-colors flex items-center gap-2 disabled:opacity-50"
              >
                {isAsking ? 'Consulting...' : <><Search className="w-3 h-3" /> Question</>}
              </button>
            </div>
            {oracleAnswer && (
              <div className="mt-4 p-4 bg-blue-900/20 border border-blue-500/20 rounded text-sm text-blue-100 leading-relaxed italic">
                "{oracleAnswer}"
              </div>
            )}
          </section>
        </div>
      </main>
    </div>
  );
}

export default App;
