import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    Terminal,
    Shield,
    Activity,
    Eye,
    Crosshair
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface ClaimStatus {
    name: string;
    status: string;
    active: boolean;
    value: string;
}

interface CortexStats {
    cpu_usage: number;
    memory_used: number;
    memory_total: number;
    uptime: number;
    firewall_active: boolean;
    logs_total: number;
    kernel_version: string;
    swarm_load: number;
    nervios_sync: boolean;
    claims: ClaimStatus[];
}

const SystemConsole: React.FC = () => {
    const [stats, setStats] = useState<CortexStats | null>(null);
    const [logs, setLogs] = useState<string[]>([]);

    const fetchData = async () => {
        try {
            const resStats = await invoke<CortexStats>('get_estadisticas_cortex');
            const rawLogs = await invoke<{ message: string; severity: string; timestamp: number }[]>('get_logs_sistema', { count: 20 });
            setStats(resStats);
            setLogs(rawLogs.map(l => `[${l.severity}] ${l.message}`));
        } catch (e) {
            console.error(e);
        }
    };

    useEffect(() => {
        fetchData();
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    return (
        <div className="p-10 space-y-8 h-full flex flex-col bg-cyber-dark text-white font-mono selection:bg-sentinel-blue/30 antialiased">
            <header className="flex justify-between items-center">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <Terminal className="text-sentinel-blue" size={20} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase font-sans">MONITOR DE <span className="text-white/20 font-sans">SISTEMA</span></h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase">Telemetría del Nodo Rust // Verificación de Seguridad</p>
                </div>

                <div className="flex items-center gap-4">
                    <div className="px-4 py-2 rounded-xl bg-white/5 border border-white/10 flex items-center gap-3">
                        <div className={`w-2 h-2 rounded-full ${stats?.nervios_sync ? 'bg-sentinel-green' : 'bg-sentinel-blue'} animate-pulse`} />
                        <span className="text-[10px] font-black uppercase tracking-widest text-white/40">Nodo Online</span>
                    </div>
                </div>
            </header>

            <div className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-8 overflow-hidden">
                {/* PANEL IZQUIERDO: MÉTRICAS CRÍTICAS */}
                <div className="lg:col-span-4 space-y-6 overflow-y-auto custom-scrollbar">
                    <div className="p-8 rounded-[2.5rem] bg-sentinel-blue/5 border border-sentinel-blue/10 space-y-8">
                        <div className="flex items-center justify-between">
                            <h3 className="text-xs font-black text-sentinel-blue uppercase tracking-widest flex items-center gap-3">
                                <Shield size={16} /> SEGURIDAD
                            </h3>
                            <span className="text-[8px] text-white/20 uppercase font-bold">Capas de Protección</span>
                        </div>

                        <div className="space-y-6">
                            {stats?.claims.map((claim, idx) => (
                                <div key={idx} className="flex items-center justify-between">
                                    <span className="text-[10px] text-white/40 uppercase">{claim.name}</span>
                                    <span className={claim.active ? 'text-sentinel-green' : 'text-white/20'}>
                                        {claim.value}
                                    </span>
                                </div>
                            ))}
                        </div>
                    </div>

                    <div className="p-8 rounded-[2.5rem] bg-white/[0.02] border border-white/5 space-y-8">
                        <h3 className="text-xs font-black text-white/40 uppercase tracking-widest flex items-center gap-3">
                            <Activity size={16} /> RECURSOS FÍSICOS
                        </h3>

                        <div className="space-y-6">
                            <div className="flex flex-col gap-2">
                                <div className="flex justify-between text-[10px] text-white/20 uppercase">
                                    <span>CPU Usage</span>
                                    <span>{stats?.cpu_usage.toFixed(1)}%</span>
                                </div>
                                <div className="h-1 bg-white/5 rounded-full overflow-hidden">
                                    <motion.div animate={{ width: `${stats?.cpu_usage}%` }} className="h-full bg-sentinel-blue" />
                                </div>
                            </div>
                            <div className="flex flex-col gap-2">
                                <div className="flex justify-between text-[10px] text-white/20 uppercase">
                                    <span>Memory usage</span>
                                    <span>{stats ? (stats.memory_used / stats.memory_total * 100).toFixed(1) : 0}%</span>
                                </div>
                                <div className="h-1 bg-white/5 rounded-full overflow-hidden">
                                    <motion.div animate={{ width: `${stats ? (stats.memory_used / stats.memory_total * 100) : 0}%` }} className="h-full bg-sentinel-green" />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* PANEL DERECHO: TERMINAL DE LOGS Y COMANDOS */}
                <div className="lg:col-span-8 flex flex-col space-y-6">
                    <div className="flex-1 bg-black/40 rounded-[2.5rem] border border-white/5 p-8 flex flex-col overflow-hidden relative group">
                        <div className="absolute top-6 right-8 flex gap-2">
                            <div className="w-2 h-2 rounded-full bg-red-400/20" />
                            <div className="w-2 h-2 rounded-full bg-yellow-400/20" />
                            <div className="w-2 h-2 rounded-full bg-green-400/20" />
                        </div>

                        <h3 className="text-[10px] font-black text-white/20 uppercase tracking-[0.4em] mb-6 flex items-center gap-3">
                            <Terminal size={14} /> INTERNAL_TELEMETRY_STREAM
                        </h3>

                        <div className="flex-1 overflow-y-auto space-y-2 text-[11px] leading-relaxed custom-scrollbar selection:bg-sentinel-blue/30 pr-4">
                            <AnimatePresence>
                                {logs.map((log, i) => (
                                    <motion.div
                                        key={i}
                                        initial={{ opacity: 0, x: -5 }}
                                        animate={{ opacity: 1, x: 0 }}
                                        className={`font-mono ${log.includes('CPU=') ? 'text-sentinel-blue/80' : 'text-white/40'}`}
                                    >
                                        <span className="text-sentinel-green/40 mr-2">➜</span>
                                        {log}
                                    </motion.div>
                                ))}
                            </AnimatePresence>
                        </div>

                        <div className="mt-6 pt-6 border-t border-white/5 flex justify-between items-center text-[9px] font-black uppercase text-white/10 tracking-widest">
                            <span>Consola de Gestión</span>
                            <div className="flex items-center gap-4">
                                <span className="flex items-center gap-2">
                                    <Eye size={12} /> TELEMETRÍA
                                </span>
                                <span className="flex items-center gap-2">
                                    <Crosshair size={12} /> ON-TARGET
                                </span>
                            </div>
                        </div>
                    </div>

                    {/* COMANDO RAPIDO */}
                    <div className="bg-sentinel-blue/5 rounded-3xl border border-sentinel-blue/10 p-4 flex items-center gap-4">
                        <div className="w-10 h-10 rounded-2xl bg-sentinel-blue/10 flex items-center justify-center text-sentinel-blue">
                            <Terminal size={18} />
                        </div>
                        <input
                            type="text"
                            placeholder="EJECUTAR COMANDO SENTINEL (EJ: sysadmin --prompt 'status')"
                            className="flex-1 bg-transparent border-none outline-none text-sentinel-blue font-black tracking-widest uppercase text-xs placeholder:text-sentinel-blue/20"
                            onKeyDown={async (e) => {
                                if (e.key === 'Enter') {
                                    const cmd = e.currentTarget.value;
                                    e.currentTarget.value = '';
                                    setLogs(prev => [`[EXEC] sentinel ${cmd}`, ...prev]);
                                    try {
                                        const res = await invoke<string>('execute_sentinel_command', { command: cmd });
                                        setLogs(prev => [res, ...prev]);
                                    } catch (err) {
                                        setLogs(prev => [`ERROR: ${err}`, ...prev]);
                                    }
                                }
                            }}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
};

export default SystemConsole;
