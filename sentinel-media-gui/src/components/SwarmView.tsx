import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    Rocket,
    Activity,
    Terminal,
    Play,
    Square,
    RefreshCw,
    Cpu,
    ShieldAlert,
    BrainCircuit,
    Settings,
    MoreHorizontal
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface AgentStatus {
    name: string;
    state: 'Idle' | 'Running' | 'Error' | 'Offline';
    description: string;
    id?: number;
    binary: string;
    agent_type: string;
}

interface CortexStats {
    swarm_load: number;
    nervios_sync: boolean;
}

const getAgentIcon = (type: string) => {
    switch (type) {
        case 'nervio': return ShieldAlert;
        case 'research': return BrainCircuit;
        case 'cortex':
        case 'media': return Terminal;
        default: return Activity;
    }
};

const SwarmView: React.FC = () => {
    const [agents, setAgents] = useState<AgentStatus[]>([]);
    const [stats, setStats] = useState<CortexStats | null>(null);
    const [loading, setLoading] = useState(true);
    const [actionLoading, setActionLoading] = useState<string | null>(null);

    const fetchData = async () => {
        try {
            const resAgents = await invoke<AgentStatus[]>('get_agentes');
            const resStats = await invoke<CortexStats>('get_estadisticas_cortex');
            setAgents(resAgents);
            setStats(resStats);
        } catch (e) {
            console.error(e);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchData();
        const interval = setInterval(fetchData, 4000);
        return () => clearInterval(interval);
    }, []);

    const handleToggle = async (agent: AgentStatus) => {
        setActionLoading(agent.name);
        try {
            if (agent.state === 'Running' && agent.id) {
                await invoke('detener_agente', { pid: agent.id });
            } else {
                await invoke('iniciar_agente', { binary: agent.binary });
            }
            setTimeout(fetchData, 800);
        } catch (e) {
            console.error(e);
        } finally {
            setActionLoading(null);
        }
    };

    return (
        <div className="p-10 space-y-10 h-full overflow-y-auto bg-cyber-dark text-white selection:bg-sentinel-blue/30 antialiased">
            {/* HEADER MODULAR */}
            <header className="flex justify-between items-center">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <Rocket className="text-sentinel-blue" size={20} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase">GESTOR <span className="text-white/20">DE AGENTES</span></h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase font-mono">Orquestación de Agentes Autónomos</p>
                </div>

                <div className="flex items-center gap-3">
                    <button
                        onClick={() => { setLoading(true); fetchData(); }}
                        className="p-3 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all text-white/40 hover:text-sentinel-blue"
                    >
                        <RefreshCw size={18} className={loading ? "animate-spin" : ""} />
                    </button>
                    <button
                        onClick={async () => {
                            setLoading(true);
                            const idleAgents = agents.filter(a => a.state !== 'Running');
                            await Promise.allSettled(idleAgents.map(a => invoke('iniciar_agente', { binary: a.binary })));
                            setTimeout(fetchData, 800);
                        }}
                        disabled={agents.every(a => a.state === 'Running')}
                        className="px-6 py-3 rounded-xl bg-sentinel-blue/10 border border-sentinel-blue/20 text-sentinel-blue text-[10px] font-black tracking-widest uppercase hover:bg-sentinel-blue hover:text-cyber-dark transition-all disabled:opacity-30 disabled:cursor-not-allowed"
                    >
                        INICIALIZAR AGENTES
                    </button>
                </div>
            </header>

            {/* GRID DE AGENTES DINÁMICO */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <AnimatePresence mode="popLayout">
                    {agents.map((agent) => {
                        const Icon = getAgentIcon(agent.agent_type);
                        return (
                            <motion.div
                                layout
                                key={agent.name}
                                initial={{ opacity: 0, y: 20 }}
                                animate={{ opacity: 1, y: 0 }}
                                exit={{ opacity: 0, scale: 0.95 }}
                                className="p-8 rounded-[2.5rem] glass-panel border border-white/5 relative overflow-hidden group hover:border-white/10 transition-all flex flex-col justify-between min-h-[220px]"
                            >
                                <div className={`absolute -top-10 -right-10 w-24 h-24 blur-[60px] opacity-20 transition-colors ${agent.state === 'Running' ? 'bg-sentinel-green' : 'bg-red-500'}`} />

                                <div className="flex justify-between items-start">
                                    <div className="flex items-center gap-5">
                                        <div className={`p-4 rounded-2xl transition-all ${agent.state === 'Running' ? 'bg-sentinel-green/10 text-sentinel-green' : 'bg-white/5 text-white/20'}`}>
                                            <Icon size={24} />
                                        </div>
                                        <div>
                                            <h3 className="text-sm font-black text-white uppercase tracking-widest mb-1">{agent.name}</h3>
                                            <div className="flex items-center gap-2">
                                                <div className={`w-1.5 h-1.5 rounded-full ${agent.state === 'Running' ? 'bg-sentinel-green animate-pulse' : 'bg-white/10'}`} />
                                                <span className="text-[9px] font-black uppercase tracking-widest text-white/30 font-mono">
                                                    {agent.state === 'Idle' ? 'LISTO' : agent.state === 'Running' ? 'EN EJECUCIÓN' : agent.state.toUpperCase()} {agent.id ? `// PID ${agent.id}` : ''}
                                                </span>
                                            </div>
                                        </div>
                                    </div>

                                    <div className="flex gap-2">
                                        <button className="p-2 text-white/10 hover:text-white/40 transition-colors">
                                            <Settings size={14} />
                                        </button>
                                        <button className="p-2 text-white/10 hover:text-white/40 transition-colors">
                                            <MoreHorizontal size={14} />
                                        </button>
                                    </div>
                                </div>

                                <div className="mt-6 mb-8">
                                    <p className="text-xs text-white/30 leading-relaxed font-medium line-clamp-2 italic">
                                        {agent.description}
                                    </p>
                                </div>

                                <div className="flex items-center justify-between pt-6 border-t border-white/5">
                                    <div className="flex items-center gap-4">
                                        <button
                                            disabled={actionLoading === agent.name}
                                            onClick={() => handleToggle(agent)}
                                            className={`flex items-center gap-2 px-5 py-2.5 rounded-xl font-black text-[9px] uppercase tracking-[0.2em] transition-all ${agent.state === 'Running'
                                                ? 'bg-red-500/10 border border-red-500/20 text-red-400 hover:bg-red-500 hover:text-white'
                                                : 'bg-sentinel-green/10 border border-sentinel-green/20 text-sentinel-green hover:bg-sentinel-green hover:text-cyber-dark'
                                                }`}
                                        >
                                            {actionLoading === agent.name ? <RefreshCw size={12} className="animate-spin" /> :
                                                agent.state === 'Running' ? <><Square size={10} fill="currentColor" /> DETENER</> :
                                                    <><Play size={10} fill="currentColor" /> INICIAR</>}
                                        </button>
                                        <span className="text-[8px] font-mono text-white/10 uppercase tracking-widest">{agent.binary}</span>
                                    </div>

                                    <div className="flex gap-1 pr-2">
                                        {[1, 2, 3].map(i => (
                                            <div key={i} className={`w-1 h-3 rounded-full ${agent.state === 'Running' ? 'bg-sentinel-green/20 animate-pulse' : 'bg-white/5'}`} style={{ animationDelay: `${i * 150}ms` }} />
                                        ))}
                                    </div>
                                </div>
                            </motion.div>
                        );
                    })}
                </AnimatePresence>
            </div>

            {/* FOOTER DE ESTADO HARDWARE DINÁMICO */}
            <footer className="p-8 rounded-[2.5rem] glass-panel border border-white/5 flex items-center justify-between relative overflow-hidden">
                <div className="absolute inset-y-0 left-0 w-1 bg-sentinel-blue shadow-[4px_0_10px_rgba(0,217,255,0.2)]" />

                <div className="flex items-center gap-8 relative z-10">
                    <div className="flex items-center gap-3">
                        <Cpu size={16} className="text-white/20" />
                        <div className="space-y-0.5">
                            <div className="text-[8px] font-black text-white/20 uppercase tracking-widest">Carga de Enjambre</div>
                            <div className="text-xs font-black text-white/60 font-mono">
                                {stats?.swarm_load ? stats.swarm_load.toFixed(2) : '0.00'} <span className="text-[10px] opacity-40 uppercase">AVG</span>
                            </div>
                        </div>
                    </div>
                    <div className="h-8 w-px bg-white/5" />
                    <div className="flex items-center gap-3">
                        {stats?.nervios_sync ? (
                            <ShieldAlert size={16} className="text-sentinel-green animate-pulse" />
                        ) : (
                            <ShieldAlert size={16} className="text-white/20" />
                        )}
                        <div className="space-y-0.5">
                            <div className="text-[8px] font-black text-white/20 uppercase tracking-widest">Nervios Activos</div>
                            <div className={`text-xs font-black font-mono ${stats?.nervios_sync ? 'text-sentinel-green' : 'text-white/40'}`}>
                                {stats?.nervios_sync ? 'A + B SYNC' : 'STANDBY'}
                            </div>
                        </div>
                    </div>
                </div>

                <div className="text-[9px] font-black text-white/10 uppercase tracking-[0.3em] flex items-center gap-2">
                    Pulsos de Sincronización <RefreshCw size={10} className="animate-spin" style={{ animationDuration: '4s' }} /> {new Date().toLocaleDateString('es-ES', { month: 'short', year: 'numeric' }).toUpperCase()}
                </div>
            </footer>
        </div>
    );
};

export default SwarmView;
