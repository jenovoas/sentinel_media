import React from 'react';
import {
    Activity,
    ShieldAlert,
    ShieldCheck,
    LayoutDashboard,
    Terminal,
    Factory
} from 'lucide-react';
import { motion } from 'framer-motion';

// Custom Hook y Componentes
import { useDashboardData, DashboardData, OpEntry, ClaimStatus } from '../hooks/useDashboardData';
import { useGpuStream } from '../hooks/useGpuStream';
import { useLogStream } from '../hooks/useLogStream';
import { KPICard } from './dashboard/KPICard';
import { ClaimCard } from './dashboard/ClaimCard';
import { HardwareMetrics } from './dashboard/HardwareMetrics';
import { SystemLogsWidget } from './dashboard/SystemLogsWidget';


interface DashboardProps {
    setView?: (view: string) => void;
}

const Dashboard: React.FC<DashboardProps> = () => {
    // 1. Polling Base (Estado General + Inicialización)
    const { state, refetch } = useDashboardData(5000);

    // Extraer datos iniciales si existen
    const initialGpu = state.status === 'success' ? state.data.gpuStatus : null;
    const initialLogs = state.status === 'success' ? state.data.systemLogs : [];

    // 2. Streams Reactivos (Overlay en tiempo real)
    const gpuStatus = useGpuStream(initialGpu);
    const systemLogs = useLogStream(initialLogs);

    // 3. Estado del agente de fábrica (ahora viene de useDashboardData)
    const factoryStatus = state.status === 'success' ? state.data.factoryStatus : { running: false, pid: null };

    // LOADING STATE
    if (state.status === 'loading') {
        return (
            <div className="p-10 h-full flex items-center justify-center bg-cyber-dark">
                <div className="text-white/40 text-sm uppercase tracking-widest animate-pulse flex flex-col items-center gap-4">
                    <Activity className="animate-spin text-sentinel-blue" size={32} />
                    Sincronizando Nodo Cortex...
                </div>
            </div>
        );
    }

    // ERROR STATE
    if (state.status === 'error') {
        return (
            <div className="p-10 h-full flex items-center justify-center bg-cyber-dark">
                <div className="p-8 rounded-4xl border border-red-500/20 bg-red-500/5 text-center max-w-md">
                    <ShieldAlert size={48} className="mx-auto mb-4 text-red-500" />
                    <h2 className="text-xl font-black text-white uppercase mb-2">Fallo Crítico de Sistema</h2>
                    <p className="text-sm text-red-400 font-mono mb-6">{state.error}</p>
                    <button
                        onClick={() => refetch()}
                        className="px-6 py-3 bg-red-500 hover:bg-red-600 text-white font-black uppercase tracking-widest text-xs rounded-xl transition-all"
                    >
                        Reintentar Conexión
                    </button>
                </div>
            </div>
        );
    }

    // SUCCESS STATE
    if (state.status !== 'success') return null;

    const data: DashboardData = state.data;

    // Métricas calculadas
    const runningCount = data.operations.filter((o: OpEntry) => o.status === 'Running' || o.status === 'Pending').length;
    const completedCount = data.operations.filter((o: OpEntry) => o.status === 'Completed' || o.status === 'Done').length;
    const failedCount = data.operations.filter((o: OpEntry) => o.status === 'Lost' || o.status === 'Failed' || o.status === 'Error').length;

    return (
        <div className="p-10 space-y-10 overflow-y-auto h-full scroll-smooth bg-cyber-dark text-white selection:bg-sentinel-blue/30 antialiased">
            {/* HEADER */}
            <header className="flex justify-between items-center">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <LayoutDashboard className="text-sentinel-blue" size={20} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase">
                            CENTRO DE <span className="text-white/20">GESTIÓN NODO</span>
                        </h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase font-mono">
                        Control de Biblioteca & Fábrica // {data.stats?.kernel_version || 'Detectando...'}
                    </p>
                </div>

                <div className="flex items-center gap-4">
                    {/* MEDIDOR DE COHERENCIA (BIO-SYNC) */}
                    <div className="flex flex-col items-end gap-1">
                        <div className="flex items-center gap-2">
                            <span className="text-[9px] font-black text-white/40 uppercase tracking-widest">Coherencia Bio-Sincrónica</span>
                            <span className={`text-[10px] font-mono font-bold ${data.stats?.coherence && data.stats.coherence > 0.9 ? 'text-sentinel-blue' : 'text-white/40'}`}>
                                {(data.stats?.coherence ? data.stats.coherence * 100 : 0).toFixed(1)}%
                            </span>
                        </div>
                        <div className="w-48 h-1.5 bg-white/5 rounded-full overflow-hidden border border-white/5 relative">
                            <motion.div 
                                className={`h-full bg-gradient-to-r ${data.stats?.coherence && data.stats.coherence > 0.9 ? 'from-sentinel-blue to-cyan-400' : 'from-white/20 to-white/40'}`}
                                initial={{ width: 0 }}
                                animate={{ width: `${(data.stats?.coherence ? data.stats.coherence * 100 : 0)}%` }}
                                transition={{ type: "spring", stiffness: 50 }}
                            />
                            {/* Glow effect if coherent */}
                            {data.stats?.coherence && data.stats.coherence > 0.9 && (
                                <div className="absolute inset-0 bg-sentinel-blue/20 blur-sm animate-pulse" />
                            )}
                        </div>
                    </div>

                    {/* Indicador de Agente Activo */}
                    {factoryStatus.running && (
                        <div className="flex items-center gap-2 px-4 py-2 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
                            <Factory size={14} className="animate-pulse" />
                            <span className="text-[10px] font-black tracking-widest uppercase">
                                Agente Activo (PID: {factoryStatus.pid})
                            </span>
                            <div className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
                        </div>
                    )}

                    {data.stats?.firewall_active ? (
                        <div className="flex items-center gap-2 px-4 py-2 rounded-xl bg-sentinel-green/10 border border-sentinel-green/20 text-sentinel-green text-[10px] font-black tracking-widest uppercase">
                            <ShieldCheck size={14} /> Filtro Activo
                        </div>
                    ) : (
                        <div className="flex items-center gap-2 px-4 py-2 rounded-xl bg-sentinel-blue/10 border border-sentinel-blue/20 text-sentinel-blue text-[10px] font-black tracking-widest uppercase">
                            <ShieldAlert size={14} /> Modo Local
                        </div>
                    )}
                </div>
            </header>

            {/* KPI CARDS */}
            <section className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <KPICard value={data.operations.length} label="Operaciones Totales" color="white" />
                <KPICard value={runningCount} label="Activas" color="blue" />
                <KPICard value={completedCount} label="Completadas" color="green" />
                <KPICard value={failedCount} label="Fallas de Sistema" color="red" />
            </section>

            {/* THE 5 CLAIMS */}
            {data.stats?.claims && (
                <section className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                    {data.stats.claims.map((claim: ClaimStatus, idx: number) => (
                        <ClaimCard key={idx} {...claim} />
                    ))}
                </section>
            )}

            <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
                {/* OPERACIONES */}
                <div className="lg:col-span-8 p-8 rounded-[2.5rem] glass flex flex-col">
                    <div className="flex justify-between items-center mb-8">
                        <h3 className="text-xs font-black text-white/60 uppercase tracking-[0.4em] flex items-center gap-3">
                            <Activity size={16} className="text-sentinel-blue" />
                            Cola de Procesamiento
                        </h3>
                        <div className="flex gap-2">
                            <span className="w-2 h-2 rounded-full bg-sentinel-green animate-pulse" />
                            <span className="text-[10px] font-bold text-sentinel-green uppercase tracking-wider">Live</span>
                        </div>
                    </div>

                    <div className="space-y-4 flex-1 overflow-y-auto custom-scrollbar pr-2 max-h-[400px]">
                        {data.operations.length === 0 ? (
                            <div className="h-full flex flex-col items-center justify-center text-white/20 py-10">
                                <Terminal size={32} className="mb-4 opacity-50" />
                                <p className="text-xs uppercase tracking-widest">Sin operaciones activas</p>
                            </div>
                        ) : (
                            data.operations.map((op: OpEntry) => (
                                <div key={op.id} className="group p-5 rounded-[1.5rem] bg-white/[0.02] border border-white/5 hover:bg-white/[0.04] transition-all relative overflow-hidden">
                                    <div className="absolute left-0 top-0 bottom-0 w-1 bg-gradient-to-b from-sentinel-blue to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                                    <div className="flex justify-between items-start mb-2">
                                        <h4 className="font-bold text-white text-sm truncate pr-4">{op.op_type}</h4>
                                        <span className={`text-[9px] font-black uppercase tracking-widest px-2 py-1 rounded-lg ${op.status === 'Running' ? 'bg-sentinel-blue/20 text-sentinel-blue' :
                                            op.status === 'Completed' ? 'bg-sentinel-green/20 text-sentinel-green' :
                                                op.status === 'Failed' ? 'bg-red-500/20 text-red-500' :
                                                    'bg-white/10 text-white/40'
                                            }`}>
                                            {op.status}
                                        </span>
                                    </div>
                                    <p className="text-xs text-white/40 mb-3 truncate font-mono">{op.target_file}</p>
                                    <div className="flex items-center gap-3">
                                        <div className="flex-1 h-1 bg-white/5 rounded-full overflow-hidden">
                                            <div className="h-full bg-sentinel-blue/50" style={{ width: `${op.progress_pct}%` }} />
                                        </div>
                                        <span className="text-[9px] font-mono text-white/30">{op.progress_pct}%</span>
                                    </div>
                                </div>
                            ))
                        )}
                    </div>
                </div>

                {/* MÉTRICAS FÍSICAS - Sidebar */}
                <div className="lg:col-span-4 space-y-6">
                    {data.stats && (
                        <HardwareMetrics
                            memoryUsed={data.stats.memory_used}
                            memoryTotal={data.stats.memory_total}
                            uptime={data.stats.uptime}
                            cpuTemp={`${data.cpuTemp}°C`}
                            gpuStatus={gpuStatus}
                        />
                    )}

                    {/* Logs Widget - Interactive */}
                    <SystemLogsWidget logs={systemLogs} />
                </div>
            </div>
        </div>
    );
};

export default Dashboard;
