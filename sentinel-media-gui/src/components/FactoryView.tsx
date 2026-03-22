import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
    Factory,
    FileCode,
    Play,
    Settings,
    CheckCircle,
    Clock,
    ArrowUpRight,
    FileVideo,
    ExternalLink,
    TrendingUp,
    Activity,
    Database,
    Zap,
    Video,
    Film,
    Wand2,
    AlertCircle,
    Server,
    RefreshCw,
    FileText,
    Sparkles,
    Cpu,
    Globe,
    Gauge,
    Square,
    PlayCircle
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface OpEntry {
    id: string;
    status: string;
    target_file: string;
    updated_at?: string;
    op_type: string;
    engine: string;
    progress_pct: number;
}

interface VaultFile {
    name: string;
    path: string;
}

interface VideoProductionStats {
    total_operations: number;
    pending: number;
    running: number;
    completed: number;
    failed: number;
    videos_ready_for_stitch: number;
    avg_generation_time_mins: number;
    active_vertex_projects: string[];
}

interface CostSummary {
    total_today: number;
    total_this_month: number;
    total_all_time: number;
    by_provider: Record<string, ProviderCostBreakdown>;
    daily_budget: number;
    monthly_budget: number;
    budget_alert_threshold: number;
    is_over_daily_budget: boolean;
    is_over_monthly_budget: boolean;
    daily_budget_usage_pct: number;
    monthly_budget_usage_pct: number;
}

interface ProviderCostBreakdown {
    today: number;
    this_month: number;
    all_time: number;
    requests_today: number;
    avg_cost_per_request: number;
}

interface CostProjection {
    current_daily_avg: number;
    projected_month_end: number;
    days_until_budget_exceeded: number | null;
    recommended_daily_limit: number;
}

// Componente de progreso circular
const CircularProgress: React.FC<{ value: number; max: number; color: string; label: string }> = ({ value, max, color, label }) => {
    const percentage = max > 0 ? (value / max) * 100 : 0;
    const circumference = 2 * Math.PI * 45;
    const strokeDashoffset = circumference - (percentage / 100) * circumference;

    return (
        <div className="relative w-32 h-32">
            <svg className="transform -rotate-90 w-32 h-32">
                <circle
                    cx="64"
                    cy="64"
                    r="45"
                    stroke="currentColor"
                    strokeWidth="8"
                    fill="transparent"
                    className="text-white/5"
                />
                <motion.circle
                    cx="64"
                    cy="64"
                    r="45"
                    stroke="currentColor"
                    strokeWidth="8"
                    fill="transparent"
                    className={color}
                    strokeDasharray={circumference}
                    initial={{ strokeDashoffset: circumference }}
                    animate={{ strokeDashoffset }}
                    transition={{ duration: 1, ease: "easeOut" }}
                    strokeLinecap="round"
                />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
                <span className="text-2xl font-black font-mono text-white">{value}</span>
                <span className="text-[8px] font-black text-white/20 uppercase tracking-widest">{label}</span>
            </div>
        </div>
    );
};

const FactoryView: React.FC = () => {
    const [ops, setOps] = useState<OpEntry[]>([]);
    const [vaultFiles, setVaultFiles] = useState<VaultFile[]>([]);
    const [stats, setStats] = useState<VideoProductionStats>({
        total_operations: 0,
        pending: 0,
        running: 0,
        completed: 0,
        failed: 0,
        videos_ready_for_stitch: 0,
        avg_generation_time_mins: 0,
        active_vertex_projects: []
    });
    const [costSummary, setCostSummary] = useState<CostSummary | null>(null);
    const [costProjection, setCostProjection] = useState<CostProjection | null>(null);
    const [gpuStatus, setGpuStatus] = useState<string>('Verificando...');
    const [factoryConfig, setFactoryConfig] = useState({
        shorts: true,
        longform: false,
        stitch: true,
        publish: false,
        local: true,
        provider: 'gemini',
        cinematic: false,
        gpu: true, // GPU activa por defecto
    });

    const [selectedFile, setSelectedFile] = useState<VaultFile | null>(null);
    const [fileContent, setFileContent] = useState<string>('');
    const [isGenerating, setIsGenerating] = useState(false);
    const [factoryAgentRunning, setFactoryAgentRunning] = useState(false);
    const [factoryAgentPid, setFactoryAgentPid] = useState<number | null>(null);

    const fetchData = async () => {
        try {
            const resOps = await invoke<OpEntry[]>('get_operaciones');
            const resVault = await invoke<VaultFile[]>('get_archivos_sentinel_media');
            const resStats = await invoke<VideoProductionStats>('get_estadisticas_fabrica');
            const resCosts = await invoke<CostSummary>('get_resumen_costos');
            const resProjection = await invoke<CostProjection>('get_cost_projection');

            setOps(resOps);
            setVaultFiles(resVault);
            setStats(resStats);
            setCostSummary(resCosts);
            setCostProjection(resProjection);
        } catch (e) {
            console.error(e);
        }
    };

    useEffect(() => {
        fetchData();
        const pollGpu = async () => {
            try {
                const status = await invoke<string>('check_gpu_status');
                setGpuStatus(status);
            } catch (e) {
                setGpuStatus('GPU APAGADA');
            }
        };

        pollGpu();
        const interval = setInterval(fetchData, 10000); // Sondeo reducido a 10s
        const gpuInterval = setInterval(pollGpu, 5000);

        // Escuchadores de eventos
        let unlistenFactoryStarted: UnlistenFn;
        let unlistenFactoryCompleted: UnlistenFn;
        let unlistenVaultUpdated: UnlistenFn;

        const setupListeners = async () => {
            unlistenFactoryStarted = await listen('generacion-fabrica-iniciada', () => {
                console.log('Evento: Generacion de fabrica iniciada');
                setIsGenerating(true);
            });

            unlistenFactoryCompleted = await listen('tarea-fabrica-completada', (event: any) => {
                console.log('Evento: Tarea de fabrica completada', event.payload);
                setIsGenerating(false);
                fetchData(); // Refrescar UI
            });

            unlistenVaultUpdated = await listen('indice-sentinel_media-actualizado', () => {
                console.log('Evento: Indice de sentinel_media actualizado');
                fetchData();
            });
        };

        setupListeners();

        // Polling del estado del agente de fábrica
        const checkFactoryAgent = async () => {
            try {
                const status = await invoke<{ running: boolean, pid: number | null }>('get_factory_status');
                setFactoryAgentRunning(status.running);
                setFactoryAgentPid(status.pid);
            } catch (e) {
                console.error('Error verificando estado del agente:', e);
            }
        };

        checkFactoryAgent();
        const factoryAgentInterval = setInterval(checkFactoryAgent, 2000);

        return () => {
            clearInterval(interval);
            clearInterval(gpuInterval);
            clearInterval(factoryAgentInterval);
            if (unlistenFactoryStarted) unlistenFactoryStarted();
            if (unlistenFactoryCompleted) unlistenFactoryCompleted();
            if (unlistenVaultUpdated) unlistenVaultUpdated();
        };
    }, []);

    // Filtro optimizado por fases
    const conceptFiles = vaultFiles.slice(0, 10);
    const generatingOps = ops.filter(o => (o.status.toLowerCase().includes('running') || o.status.toLowerCase().includes('pending')) && o.op_type.toLowerCase().includes('generation'));
    const processingOps = ops.filter(o => (o.status.toLowerCase().includes('running') || o.status.toLowerCase().includes('pending')) && o.op_type.toLowerCase().includes('stitch'));
    const publishedOps = ops.filter(o => o.status === 'Completed' || o.status === 'Done');
    const recentOps = ops.slice(0, 5);

    // Calcular tasa de exito
    const totalProcessed = stats.completed + stats.failed;
    const successRate = totalProcessed > 0 ? ((stats.completed / totalProcessed) * 100).toFixed(1) : '0.0';

    return (
        <div className="p-6 md:p-10 space-y-8 h-full overflow-y-auto flex flex-col bg-cyber-dark text-white selection:bg-sentinel-blue/30 antialiased custom-scrollbar">
            <header className="flex justify-between items-end">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <Factory className="text-sentinel-blue" size={20} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase">PIPELINE DE <span className="text-white/20">FÁBRICA</span></h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase font-mono">Orquestación Multimedia Local // Producción Activa</p>
                </div>

                <div className="flex items-center gap-4">
                    <button
                        onClick={async () => {
                            try {
                                setIsGenerating(true); // Reutilizamos el spinner para feedback
                                await invoke('escanear_sentinel_media_fabrica');
                                fetchData();
                                // Pequeño delay visual para confirmar
                                setTimeout(() => setIsGenerating(false), 1000);
                            } catch (e) {
                                console.error("Error en scan:", e);
                                setIsGenerating(false);
                            }
                        }}
                        className="flex items-center gap-2 px-6 py-3 rounded-xl bg-white/5 border border-white/10 text-white/40 text-[10px] font-black tracking-widest uppercase hover:bg-white/10 transition-all active:scale-95 group"
                    >
                        <RefreshCw size={14} className={isGenerating ? "animate-spin text-sentinel-blue" : "group-hover:rotate-180 transition-transform duration-500"} /> ESCANEAR BÓVEDA
                    </button>
                    <button
                        onClick={async () => {
                            try {
                                setIsGenerating(true);
                                await invoke('ejecutar_generacion_fabrica', { config: factoryConfig });
                                fetchData();
                            } catch (e) {
                                console.error("Error en fábrica:", e);
                            } finally {
                                setIsGenerating(false);
                            }
                        }}
                        disabled={isGenerating}
                        className="flex items-center gap-2 px-6 py-3 rounded-xl bg-sentinel-blue/10 border border-sentinel-blue/20 text-sentinel-blue text-[10px] font-black tracking-widest uppercase hover:bg-sentinel-blue hover:text-cyber-dark transition-all active:scale-95 disabled:opacity-50"
                    >
                        <Zap size={14} fill="currentColor" className={isGenerating ? 'animate-spin' : ''} />
                        {isGenerating ? 'INICIANDO...' : 'EJECUTAR FÁBRICA'}
                    </button>

                    {/* Botones de control del agente de fábrica */}
                    {!factoryAgentRunning ? (
                        <button
                            onClick={async () => {
                                try {
                                    const result = await invoke<string>('run_factory_agent');
                                    console.log('Agente iniciado:', result);
                                } catch (e) {
                                    console.error('Error iniciando agente:', e);
                                    alert(`Error: ${e}`);
                                }
                            }}
                            className="flex items-center gap-2 px-6 py-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-[10px] font-black tracking-widest uppercase hover:bg-emerald-500/20 transition-all active:scale-95"
                        >
                            <PlayCircle size={14} /> INICIAR DAEMON
                        </button>
                    ) : (
                        <button
                            onClick={async () => {
                                try {
                                    const result = await invoke<string>('stop_factory_agent');
                                    console.log('Agente detenido:', result);
                                } catch (e) {
                                    console.error('Error deteniendo agente:', e);
                                    alert(`Error: ${e}`);
                                }
                            }}
                            className="flex items-center gap-2 px-6 py-3 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-[10px] font-black tracking-widest uppercase hover:bg-red-500/20 transition-all active:scale-95"
                        >
                            <Square size={14} fill="currentColor" /> DETENER DAEMON (PID: {factoryAgentPid})
                        </button>
                    )}
                </div>
            </header>

            {/* PANEL DE CONFIGURACIÓN DE FÁBRICA */}
            <section className="p-8 rounded-[2.5rem] bg-white/[0.01] border border-white/5 space-y-6">
                <div className="flex flex-col md:flex-row gap-8">
                    {/* Configuración de Modo */}
                    <div className="flex-1 space-y-6">
                        <div className="flex items-center gap-3">
                            <Settings size={16} className="text-white/40" />
                            <h3 className="text-xs font-black text-white/40 uppercase tracking-widest">Modo de Ejecución</h3>
                        </div>
                        <div className="flex flex-wrap gap-4">
                            {[
                                { id: 'shorts', label: 'SHORTS', icon: <Video size={16} />, desc: 'Vertical' },
                                { id: 'longform', label: 'LONGFORM', icon: <Film size={16} />, desc: 'Horizontal' },
                                { id: 'stitch', label: 'STITCH', icon: <Database size={16} />, desc: 'Ensamblar' },
                                { id: 'publish', label: 'PUBLISH', icon: <ArrowUpRight size={16} />, desc: 'YouTube' },
                                { id: 'local', label: 'LOCAL', icon: <Server size={16} />, desc: 'Dev Mode' },
                                { id: 'cinematic', label: 'CINEMATIC', icon: <Wand2 size={16} />, desc: 'Remotion Engine' },
                                { id: 'gpu', label: 'GPU ACCEL', icon: <Zap size={16} />, desc: 'Hardware NVENC' },
                            ].map((cfg) => (
                                <div key={cfg.id} className="relative group">
                                    <button
                                        onClick={() => setFactoryConfig(prev => ({ ...prev, [cfg.id]: !prev[cfg.id as keyof typeof factoryConfig] }))}
                                        className={`flex flex-col p-5 rounded-2xl border transition-all min-w-[140px] cursor-pointer group active:scale-95 ${factoryConfig[cfg.id as keyof typeof factoryConfig]
                                            ? cfg.id === 'cinematic'
                                                ? 'bg-purple-500/20 border-purple-500/40 text-purple-400 shadow-[0_0_20px_rgba(168,85,247,0.2)]'
                                                : cfg.id === 'gpu'
                                                    ? 'bg-orange-500/20 border-orange-500/40 text-orange-400 shadow-[0_0_20px_rgba(249,115,22,0.2)]'
                                                    : 'bg-sentinel-blue/10 border-sentinel-blue/20 text-sentinel-blue shadow-[0_0_20px_rgba(0,217,255,0.1)]'
                                            : 'bg-white/[0.02] border-white/5 text-white/20 hover:border-white/10 hover:bg-white/[0.04]'
                                            }`}
                                    >
                                        <div className="flex items-center gap-3 mb-2">
                                            <div className={`p-2 rounded-lg transition-colors ${factoryConfig[cfg.id as keyof typeof factoryConfig] ? 'bg-current/10' : 'bg-white/5'}`}>
                                                {cfg.icon}
                                            </div>
                                            <span className="text-[11px] font-black tracking-widest">{cfg.label}</span>
                                        </div>
                                        <span className="text-[9px] font-bold opacity-40 uppercase tracking-tight">{cfg.desc}</span>
                                    </button>

                                    {cfg.id === 'gpu' && factoryConfig.gpu && (
                                        <div className="absolute -bottom-10 left-0 right-0 text-center pointer-events-none">
                                            <span className="text-[7px] font-mono text-orange-400/60 uppercase tracking-tighter whitespace-nowrap bg-black/40 px-2 py-1 rounded-md border border-orange-500/10">
                                                {gpuStatus}
                                            </span>
                                        </div>
                                    )}
                                </div>
                            ))}
                        </div>
                    </div>

                    {/* Selector de Proveedor */}
                    <div className="border-l border-white/5 pl-8 space-y-6">
                        <div className="flex items-center gap-3">
                            <Zap size={16} className="text-white/40" />
                            <h3 className="text-xs font-black text-white/40 uppercase tracking-widest">Motor de IA</h3>
                        </div>
                        <div className="grid grid-cols-2 gap-3">
                            {[
                                { id: 'gemini', label: 'Gemini 2.0', sub: 'Deep Research', icon: <Sparkles size={14} />, color: 'text-sentinel-blue', border: 'border-sentinel-blue/20' },
                                { id: 'groq', label: 'Groq Lr3', sub: 'Ultra Fast', icon: <Zap size={14} />, color: 'text-orange-400', border: 'border-orange-400/20' },
                                { id: 'openai', label: 'OpenAI Pro', sub: 'GPT-4o', icon: <Cpu size={14} />, color: 'text-sentinel-green', border: 'border-sentinel-green/20' },
                                { id: 'antigravity', label: 'Antigravity', sub: 'Internal', icon: <Database size={14} />, color: 'text-purple-400', border: 'border-purple-400/20' },
                                { id: 'perplexity', label: 'Perplexity', sub: 'Sonar Pro', icon: <Globe size={14} />, color: 'text-cyan-400', border: 'border-cyan-400/20' },
                            ].map((prov) => (
                                <button
                                    key={prov.id}
                                    onClick={() => setFactoryConfig(prev => ({ ...prev, provider: prov.id }))}
                                    className={`flex flex-col p-4 rounded-xl border transition-all text-left group active:scale-95 cursor-pointer relative overflow-hidden ${factoryConfig.provider === prov.id
                                        ? `bg-white/[0.05] ${prov.border} ${prov.color} shadow-[0_0_15px_rgba(255,255,255,0.05)]`
                                        : 'bg-white/[0.01] border-white/5 text-white/20 hover:border-white/10'
                                        }`}
                                >
                                    <div className="flex items-center gap-2 mb-1">
                                        <div className={`p-1.5 rounded-lg transition-colors ${factoryConfig.provider === prov.id ? 'bg-current/10' : 'bg-white/5'}`}>
                                            {prov.icon}
                                        </div>
                                        <span className="text-[10px] font-black tracking-widest uppercase">{prov.label}</span>
                                    </div>
                                    <span className="text-[7px] font-mono opacity-40 group-hover:opacity-60 transition-opacity pl-8">{prov.sub}</span>

                                    {factoryConfig.provider === prov.id && (
                                        <motion.div
                                            layoutId="activeProviderIndicator"
                                            className={`absolute bottom-0 left-0 h-0.5 bg-current w-full`}
                                        />
                                    )}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>
            </section>

            {/* DASHBOARD DE ESTADÍSTICAS AVANZADAS */}
            <section className="grid grid-cols-1 lg:grid-cols-12 gap-6">
                {/* PANEL IZQUIERDO: GRÁFICOS CIRCULARES */}
                <div className="lg:col-span-4 space-y-6">
                    {/* Circular Progress Cards */}
                    <div className="p-8 rounded-[2.5rem] glass-panel border border-white/5 space-y-8">
                        <div className="flex items-center justify-between">
                            <h3 className="text-xs font-black text-white/40 uppercase tracking-widest flex items-center gap-3">
                                <Activity size={16} /> ESTADO DE PRODUCCIÓN
                            </h3>
                            <span className="text-[8px] text-white/10 uppercase font-bold">En Tiempo Real</span>
                        </div>

                        <div className="grid grid-cols-2 gap-6">
                            <div className="flex flex-col items-center space-y-3">
                                <CircularProgress
                                    value={stats.running}
                                    max={stats.total_operations}
                                    color="text-sentinel-blue"
                                    label="Activos"
                                />
                                <div className="text-center">
                                    <div className="text-[10px] font-black text-sentinel-blue uppercase tracking-widest">Generando</div>
                                    <div className="text-[8px] text-white/20 font-mono">{stats.running}/{stats.total_operations}</div>
                                </div>
                            </div>

                            <div className="flex flex-col items-center space-y-3">
                                <CircularProgress
                                    value={stats.completed}
                                    max={stats.total_operations}
                                    color="text-sentinel-green"
                                    label="Listos"
                                />
                                <div className="text-center">
                                    <div className="text-[10px] font-black text-sentinel-green uppercase tracking-widest">Completados</div>
                                    <div className="text-[8px] text-white/20 font-mono">{stats.completed}/{stats.total_operations}</div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* KPIs Card */}
                    <div className="p-8 rounded-[2.5rem] bg-gradient-to-br from-sentinel-blue/10 to-transparent border border-sentinel-blue/20 space-y-6">
                        <div className="flex items-center gap-3">
                            <TrendingUp size={20} className="text-sentinel-blue" />
                            <h3 className="text-xs font-black text-white uppercase tracking-widest">MÉTRICAS CLAVE</h3>
                        </div>

                        <div className="space-y-4">
                            <div className="flex justify-between items-center">
                                <span className="text-[10px] text-white/40 uppercase tracking-widest">Tasa de Éxito</span>
                                <span className="text-xl font-black font-mono text-sentinel-green">{successRate}%</span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-[10px] text-white/40 uppercase tracking-widest">Tiempo Promedio</span>
                                <span className="text-xl font-black font-mono text-sentinel-blue">{stats.avg_generation_time_mins.toFixed(1)}m</span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-[10px] text-white/40 uppercase tracking-widest">Videos para Stitch</span>
                                <span className="text-xl font-black font-mono text-white/60">{stats.videos_ready_for_stitch}</span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-[10px] text-white/40 uppercase tracking-widest">En Cola</span>
                                <span className="text-xl font-black font-mono text-white/40">{stats.pending}</span>
                            </div>
                            {stats.failed > 0 && (
                                <div className="flex justify-between items-center">
                                    <span className="text-[10px] text-white/40 uppercase tracking-widest flex items-center gap-2">
                                        <AlertCircle size={12} className="text-red-400" /> Fallidos
                                    </span>
                                    <span className="text-xl font-black font-mono text-red-400">{stats.failed}</span>
                                </div>
                            )}
                        </div>
                    </div>
                </div>

                {/* PANEL CENTRAL: TIMELINE DE OPERACIONES */}
                <div className="lg:col-span-5 space-y-6">
                    {selectedFile ? (
                        <div className="p-8 rounded-[2.5rem] bg-sentinel-blue/5 border border-sentinel-blue/20 h-full flex flex-col relative group">
                            <button
                                onClick={() => setSelectedFile(null)}
                                className="absolute top-6 right-6 p-2 rounded-xl bg-white/5 hover:bg-white/10 text-white/40 hover:text-white transition-all"
                            >
                                <ArrowUpRight size={14} className="rotate-45" />
                            </button>

                            <div className="flex items-center gap-3 mb-6">
                                <FileText size={16} className="text-sentinel-blue" />
                                <h3 className="text-xs font-black text-white uppercase tracking-widest">Vista Previa: {selectedFile.name}</h3>
                            </div>

                            <div className="flex-1 overflow-y-auto custom-scrollbar bg-black/40 rounded-2xl p-6 relative">
                                <pre className="text-[10px] font-mono text-white/60 whitespace-pre-wrap leading-relaxed">
                                    {fileContent}
                                </pre>

                                <div className="absolute inset-x-0 bottom-0 h-20 bg-gradient-to-t from-black/60 to-transparent pointer-events-none" />
                            </div>

                            <div className="mt-6 flex gap-4">
                                <button
                                    onClick={() => invoke('ejecutar_generacion_fabrica', { config: { ...factoryConfig, specific_file: selectedFile.path } })}
                                    className="flex-1 py-4 bg-sentinel-blue text-cyber-dark text-[10px] font-black uppercase tracking-widest rounded-xl hover:shadow-[0_0_20px_rgba(0,217,255,0.4)] transition-all flex items-center justify-center gap-2"
                                >
                                    <Play size={14} fill="currentColor" /> Procesar este Script
                                </button>
                            </div>
                        </div>
                    ) : (
                        <div className="p-8 rounded-[2.5rem] bg-white/[0.01] border border-white/5 h-full flex flex-col">
                            <div className="flex items-center justify-between mb-6">
                                <h3 className="text-xs font-black text-white/40 uppercase tracking-widest flex items-center gap-3">
                                    <Clock size={16} /> TIMELINE DE PRODUCCIÓN
                                </h3>
                                <span className="text-[8px] text-white/10 uppercase font-bold">Últimas 5 Operaciones</span>
                            </div>

                            <div className="flex-1 space-y-4 overflow-y-auto custom-scrollbar">
                                <AnimatePresence>
                                    {recentOps.length === 0 ? (
                                        <div className="h-full flex flex-col items-center justify-center opacity-10 space-y-4">
                                            <FileVideo size={48} />
                                            <span className="text-xs font-black tracking-widest uppercase text-center">
                                                No hay operaciones<br />en el historial
                                            </span>
                                        </div>
                                    ) : (
                                        recentOps.map((op, idx) => {
                                            const statusColor =
                                                op.status === 'Running' ? 'border-sentinel-blue bg-sentinel-blue/5' :
                                                    op.status === 'Completed' || op.status === 'Done' ? 'border-sentinel-green bg-sentinel-green/5' :
                                                        op.status === 'Pending' ? 'border-white/10 bg-white/[0.02]' :
                                                            'border-red-400/20 bg-red-400/5';

                                            const statusIcon =
                                                op.status === 'Running' ? <Zap size={14} className="text-sentinel-blue animate-pulse" /> :
                                                    op.status === 'Completed' || op.status === 'Done' ? <CheckCircle size={14} className="text-sentinel-green" /> :
                                                        op.status === 'Pending' ? <Clock size={14} className="text-white/40" /> :
                                                            <AlertCircle size={14} className="text-red-400" />;

                                            return (
                                                <motion.div
                                                    key={op.id}
                                                    initial={{ opacity: 0, x: -20 }}
                                                    animate={{ opacity: 1, x: 0 }}
                                                    transition={{ delay: idx * 0.1 }}
                                                    className={`p-4 rounded-2xl border ${statusColor} space-y-3`}
                                                >
                                                    <div className="flex items-start justify-between">
                                                        <div className="flex items-center gap-2">
                                                            {statusIcon}
                                                            <span className="text-[10px] font-black text-white/80 uppercase tracking-tight line-clamp-1">
                                                                {op.target_file}
                                                            </span>
                                                        </div>
                                                        <span className="text-[8px] font-mono text-white/20 uppercase">{op.status}</span>
                                                    </div>
                                                    {op.updated_at && (
                                                        <div className="text-[8px] text-white/10 font-mono">
                                                            {new Date(op.updated_at).toLocaleString('es-CL', {
                                                                hour: '2-digit',
                                                                minute: '2-digit',
                                                                day: '2-digit',
                                                                month: 'short'
                                                            })}
                                                        </div>
                                                    )}
                                                </motion.div>
                                            );
                                        })
                                    )}
                                </AnimatePresence>
                            </div>
                        </div>
                    )}
                </div>

                {/* PANEL DERECHO: VERTEX AI PROJECTS */}
                <div className="lg:col-span-3 space-y-6">
                    <div className="p-8 rounded-[2.5rem] bg-white/[0.01] border border-white/5 space-y-6">
                        <div className="flex items-center gap-3">
                            <Server size={16} className="text-sentinel-blue" />
                            <h3 className="text-xs font-black text-white/40 uppercase tracking-widest">Vertex AI</h3>
                        </div>

                        {stats.active_vertex_projects.length > 0 ? (
                            <div className="space-y-4">
                                <div className="text-[10px] text-white/20 uppercase tracking-widest">
                                    Proyectos Activos ({stats.active_vertex_projects.length})
                                </div>
                                {stats.active_vertex_projects.map((proj, idx) => (
                                    <motion.div
                                        key={idx}
                                        initial={{ opacity: 0, scale: 0.95 }}
                                        animate={{ opacity: 1, scale: 1 }}
                                        whileHover={{ scale: 1.02 }}
                                        whileTap={{ scale: 0.98 }}
                                        className="p-4 rounded-2xl bg-sentinel-blue/10 border border-sentinel-blue/20 space-y-2 cursor-pointer group active:bg-sentinel-blue/20 transition-colors"
                                        onClick={() => {
                                            // TODO: disparador real de health check
                                            console.log(`Verificando salud del proyecto Vertex: ${proj}`);
                                        }}
                                    >
                                        <div className="flex items-center justify-between">
                                            <span className="text-[9px] font-black text-sentinel-blue uppercase tracking-widest">
                                                Proyecto #{idx + 1}
                                            </span>
                                            <div className="flex items-center gap-2">
                                                <span className="text-[7px] font-black text-sentinel-blue/40 uppercase tracking-tighter opacity-0 group-hover:opacity-100 transition-opacity">Health OK</span>
                                                <div className="w-2 h-2 rounded-full bg-sentinel-blue animate-pulse" />
                                            </div>
                                        </div>
                                        <div className="text-[10px] font-mono text-white/60 break-all">
                                            {proj}
                                        </div>
                                    </motion.div>
                                ))}
                                <div className="pt-4 border-t border-white/5 text-[8px] text-white/10 uppercase tracking-widest text-center">
                                    Round-Robin Load Balancing
                                </div>
                            </div>
                        ) : (
                            <div className="flex flex-col items-center justify-center py-8 opacity-10 space-y-3">
                                <Server size={32} />
                                <span className="text-[8px] font-black tracking-widest uppercase text-center">
                                    No hay proyectos<br />configurados
                                </span>
                            </div>
                        )}
                    </div>

                    {/* PRODUCTION WORKLOAD */}
                    <div className="p-8 rounded-[2.5rem] bg-white/[0.01] border border-white/5 space-y-6">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <Gauge size={16} className="text-sentinel-blue" />
                                <h3 className="text-xs font-black text-white/40 uppercase tracking-widest">Carga de Producción</h3>
                            </div>
                            <span className="text-[10px] font-mono text-sentinel-blue">
                                {Math.round(((stats.running + stats.pending) / (stats.total_operations || 1)) * 100)}%
                            </span>
                        </div>

                        <div className="h-2 w-full bg-white/5 rounded-full overflow-hidden relative">
                            <motion.div
                                initial={{ width: 0 }}
                                animate={{ width: `${Math.min(100, ((stats.running + stats.pending) / (stats.total_operations || 1)) * 100)}%` }}
                                className="absolute top-0 left-0 h-full bg-sentinel-blue shadow-[0_0_10px_rgba(0,217,255,0.5)]"
                            />
                            <motion.div
                                initial={{ width: 0 }}
                                animate={{ width: `${Math.min(100, (stats.running / (stats.total_operations || 1)) * 100)}%` }}
                                className="absolute top-0 left-0 h-full bg-sentinel-green/40"
                            />
                        </div>

                        <div className="flex justify-between text-[8px] font-black uppercase tracking-widest text-white/20">
                            <div className="flex items-center gap-1.5">
                                <div className="w-1.5 h-1.5 rounded-full bg-sentinel-green/40" /> Ejecutando ({stats.running})
                            </div>
                            <div className="flex items-center gap-1.5">
                                <div className="w-1.5 h-1.5 rounded-full bg-sentinel-blue" /> Total Queue ({stats.pending + stats.running})
                            </div>
                        </div>
                    </div>

                    {/* Quick Stats Mini Cards */}
                    <div className="grid grid-cols-2 gap-4">
                        <div className="p-4 rounded-2xl bg-white/[0.02] border border-white/5 text-center">
                            <div className="text-2xl font-black text-white font-mono">{stats.total_operations}</div>
                            <div className="text-[8px] font-black text-white/20 uppercase tracking-widest mt-1">Total Ops</div>
                        </div>
                        <div className="p-4 rounded-2xl bg-white/[0.02] border border-white/5 text-center">
                            <div className="text-2xl font-black text-sentinel-blue font-mono">{stats.running}</div>
                            <div className="text-[8px] font-black text-white/20 uppercase tracking-widest mt-1">Activos</div>
                        </div>
                    </div>
                </div>
            </section>

            {/* PANEL DE COSTOS Y PRESUPUESTO */}
            {costSummary && (
                <section className="grid grid-cols-1 xl:grid-cols-12 gap-6">
                    {/* Resumen de Costos */}
                    <div className="xl:col-span-8 p-6 md:p-8 rounded-[2.5rem] bg-gradient-to-br from-yellow-500/10 to-transparent border border-yellow-500/20 space-y-6">
                        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                            <div className="flex items-center gap-3">
                                <div className="w-10 h-10 rounded-2xl bg-yellow-500/20 flex items-center justify-center shrink-0">
                                    <TrendingUp size={20} className="text-yellow-500" />
                                </div>
                                <div>
                                    <h3 className="text-xs font-black text-white uppercase tracking-widest">COSTOS DE PRODUCCIÓN</h3>
                                    <p className="text-[8px] text-white/20 uppercase tracking-widest font-mono">Control de Gastos en Tiempo Real</p>
                                </div>
                            </div>
                            {(costSummary?.is_over_daily_budget || costSummary?.is_over_monthly_budget) && (
                                <div className="flex items-center gap-2 px-4 py-2 rounded-xl bg-red-500/20 border border-red-500/40">
                                    <AlertCircle size={16} className="text-red-400 animate-pulse" />
                                    <span className="text-[10px] font-black text-red-400 uppercase tracking-widest">
                                        {costSummary?.is_over_daily_budget ? 'Presupuesto Diario Excedido' : 'Presupuesto Mensual Excedido'}
                                    </span>
                                </div>
                            )}
                        </div>

                        <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 md:gap-4">
                            <div className="p-4 rounded-2xl bg-white/[0.02] border border-white/5 text-center">
                                <div className="text-[10px] text-white/40 uppercase tracking-widest mb-2">Hoy</div>
                                <div className="text-xl md:text-2xl font-black font-mono text-yellow-500">${costSummary.total_today.toFixed(2)}</div>
                                {costSummary?.daily_budget > 0 && (
                                    <div className="mt-2 text-[8px] text-white/20">
                                        {costSummary?.daily_budget_usage_pct.toFixed(0)}% de ${costSummary?.daily_budget}
                                    </div>
                                )}
                            </div>

                            <div className="p-4 rounded-2xl bg-white/[0.02] border border-white/5 text-center">
                                <div className="text-[10px] text-white/40 uppercase tracking-widest mb-2">Este Mes</div>
                                <div className="text-xl md:text-2xl font-black font-mono text-white">${costSummary.total_this_month.toFixed(2)}</div>
                                {costSummary?.monthly_budget > 0 && (
                                    <div className="mt-2 text-[8px] text-white/20">
                                        {costSummary?.monthly_budget_usage_pct.toFixed(0)}% de ${costSummary?.monthly_budget}
                                    </div>
                                )}
                            </div>

                            <div className="p-4 rounded-2xl bg-white/[0.02] border border-white/5 text-center">
                                <div className="text-[10px] text-white/40 uppercase tracking-widest mb-2">Proyección</div>
                                <div className="text-xl md:text-2xl font-black font-mono text-sentinel-blue">
                                    ${costProjection?.projected_month_end.toFixed(2) || '0.00'}
                                </div>
                                <div className="mt-2 text-[8px] text-white/20">Fin de mes</div>
                            </div>

                            <div className="p-4 rounded-2xl bg-white/[0.02] border border-white/5 text-center">
                                <div className="text-[10px] text-white/40 uppercase tracking-widest mb-2">Promedio/Día</div>
                                <div className="text-xl md:text-2xl font-black font-mono text-sentinel-green">
                                    ${costProjection?.current_daily_avg.toFixed(2) || '0.00'}
                                </div>
                                {costProjection && costProjection.days_until_budget_exceeded !== null && costProjection.days_until_budget_exceeded !== undefined && (
                                    <div className="mt-2 text-[8px] text-red-400">
                                        {costProjection.days_until_budget_exceeded === 0
                                            ? 'Presupuesto excedido'
                                            : `${costProjection.days_until_budget_exceeded} días restantes`}
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* Desglose por Proveedor */}
                        <div className="space-y-3">
                            <div className="text-[10px] font-black text-white/40 uppercase tracking-widest">Desglose por Proveedor</div>
                            {Object.entries(costSummary.by_provider).map(([provider, breakdown]) => (
                                <div key={provider} className="flex items-center justify-between p-3 rounded-xl bg-white/[0.02] border border-white/5">
                                    <div className="flex items-center gap-3">
                                        <Server size={14} className="text-sentinel-blue" />
                                        <div>
                                            <div className="text-[10px] font-black text-white uppercase tracking-tight">{provider}</div>
                                            <div className="text-[8px] text-white/20 font-mono">{breakdown.requests_today} requests hoy</div>
                                        </div>
                                    </div>
                                    <div className="text-right">
                                        <div className="text-sm font-black font-mono text-yellow-500">${breakdown.today.toFixed(2)}</div>
                                        <div className="text-[8px] text-white/20">${breakdown.avg_cost_per_request.toFixed(4)}/req</div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>

                    {/* Alertas y Recomendaciones */}
                    <div className="xl:col-span-4 space-y-6">
                        {/* Budget Progress */}
                        {costSummary?.monthly_budget > 0 && (
                            <div className="p-6 md:p-8 rounded-[2.5rem] bg-white/[0.01] border border-white/5 space-y-6">
                                <h3 className="text-xs font-black text-white/40 uppercase tracking-widest">Presupuesto Mensual</h3>

                                <div className="relative pt-1">
                                    <div className="flex mb-2 items-center justify-between">
                                        <div>
                                            <span className="text-xs font-semibold inline-block text-white">
                                                ${costSummary?.total_this_month.toFixed(2)}
                                            </span>
                                        </div>
                                        <div className="text-right">
                                            <span className="text-xs font-semibold inline-block text-white/40">
                                                ${costSummary?.monthly_budget.toFixed(2)}
                                            </span>
                                        </div>
                                    </div>
                                    <div className="overflow-hidden h-2 text-xs flex rounded-full bg-white/5">
                                        <motion.div
                                            initial={{ width: 0 }}
                                            animate={{ width: `${Math.min(costSummary?.monthly_budget_usage_pct, 100)}%` }}
                                            transition={{ duration: 1, ease: "easeOut" }}
                                            className={`shadow-none flex flex-col text-center whitespace-nowrap text-white justify-center ${costSummary?.monthly_budget_usage_pct > 90 ? 'bg-red-500' :
                                                costSummary?.monthly_budget_usage_pct > 75 ? 'bg-yellow-500' :
                                                    'bg-sentinel-green'
                                                }`}
                                        />
                                    </div>
                                    <div className="mt-2 text-center text-[10px] font-black text-white/20 uppercase tracking-widest">
                                        {costSummary?.monthly_budget_usage_pct.toFixed(1)}% Utilizado
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Recomendaciones */}
                        {costProjection && (
                            <div className="p-6 md:p-8 rounded-[2.5rem] bg-sentinel-blue/5 border border-sentinel-blue/20 space-y-4">
                                <div className="flex items-center gap-2">
                                    <TrendingUp size={16} className="text-sentinel-blue" />
                                    <h3 className="text-xs font-black text-sentinel-blue uppercase tracking-widest">Optimización</h3>
                                </div>

                                <div className="space-y-3">
                                    <div className="flex justify-between items-center gap-2">
                                        <span className="text-[10px] text-white/40 uppercase tracking-widest">Límite Diario Ideal</span>
                                        <span className="text-sm font-black font-mono text-sentinel-blue">
                                            ${costProjection?.recommended_daily_limit.toFixed(2) || '0.00'}
                                        </span>
                                    </div>

                                    {costProjection && costProjection.current_daily_avg > costProjection.recommended_daily_limit && (
                                        <div className="p-3 rounded-xl bg-yellow-500/10 border border-yellow-500/20">
                                            <div className="flex items-start gap-2">
                                                <AlertCircle size={14} className="text-yellow-500 mt-0.5 shrink-0" />
                                                <div className="text-[9px] text-yellow-500 leading-relaxed">
                                                    Estás gastando <strong>${(costProjection.current_daily_avg - costProjection.recommended_daily_limit).toFixed(2)}/día</strong> por encima del límite recomendado.
                                                </div>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}
                    </div>
                </section>
            )}

            {/* INFO DE VERTEX PROJECTS (Deprecated - moved to sidebar) */}
            {/* {stats.active_vertex_projects.length > 0 && (
                <div className="flex items-center gap-3 px-4 py-2 rounded-xl bg-white/5 border border-white/10 text-[10px] font-mono text-white/40">
                    <span className="font-black uppercase tracking-widest">Vertex Projects:</span>
                    {stats.active_vertex_projects.map((proj, idx) => (
                        <span key={idx} className="px-3 py-1 rounded-lg bg-sentinel-blue/10 text-sentinel-blue font-bold">
                            {proj}
                        </span>
                    ))}
                </div>
            )} */}

            <div className="flex-1 grid grid-cols-1 lg:grid-cols-4 gap-6 overflow-hidden">
                {/* FASE 1: CONCEPTO */}
                <div className="flex flex-col space-y-4">
                    <div className="flex items-center justify-between px-2">
                        <span className="text-[10px] font-black text-white/40 uppercase tracking-[0.2em]">1. Concepto</span>
                        <FileCode size={14} className="text-white/20" />
                    </div>
                    <div className="flex-1 bg-white/[0.01] border border-white/5 rounded-[2.5rem] p-6 space-y-4 overflow-y-auto custom-scrollbar">
                        {conceptFiles.map((file, idx) => (
                            <div
                                key={idx}
                                onClick={async () => {
                                    setSelectedFile(file);
                                    try {
                                        const content = await invoke<string>('read_vault_file_content', { path: file.path });
                                        setFileContent(content);
                                    } catch (e) {
                                        setFileContent("Error leyendo archivo: " + e);
                                    }
                                }}
                                className={`p-4 rounded-2xl border transition-all group cursor-pointer active:scale-95 ${selectedFile?.path === file.path ? 'bg-sentinel-blue/10 border-sentinel-blue/40' : 'bg-white/[0.02] border-white/5 hover:border-sentinel-blue/40 hover:bg-white/[0.04]'}`}
                            >
                                <div className={`text-[10px] font-black uppercase tracking-tight line-clamp-1 transition-colors ${selectedFile?.path === file.path ? 'text-sentinel-blue' : 'text-white/60 group-hover:text-sentinel-blue'}`}>{file.name.replace('.md', '')}</div>
                                <div className="flex justify-between items-center mt-3">
                                    <span className="text-[8px] font-mono text-white/10 uppercase group-hover:text-white/30">Script MD</span>
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            invoke('ejecutar_generacion_fabrica', { config: { ...factoryConfig, specific_file: file.path } });
                                        }}
                                        className="text-sentinel-blue opacity-40 group-hover:opacity-100 transition-opacity p-1 hover:bg-sentinel-blue/10 rounded-lg"
                                    >
                                        <Play size={12} fill="currentColor" />
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* FASE 2: GENERACIÓN */}
                <div className="flex flex-col space-y-4">
                    <div className="flex items-center justify-between px-2">
                        <span className="text-[10px] font-black text-sentinel-blue uppercase tracking-[0.2em]">2. Generación</span>
                        <Zap size={14} className="text-sentinel-blue animate-pulse" />
                    </div>
                    <div className="flex-1 bg-sentinel-blue/[0.02] border border-sentinel-blue/10 rounded-[2.5rem] p-6 space-y-4 overflow-y-auto custom-scrollbar">
                        <AnimatePresence>
                            {generatingOps.length === 0 ? (
                                <div className="h-full flex flex-col items-center justify-center opacity-10 space-y-2">
                                    <Clock size={24} />
                                    <span className="text-[8px] font-black tracking-widest uppercase">Sin procesos</span>
                                </div>
                            ) : (
                                generatingOps.map((op) => (
                                    <motion.div
                                        key={op.id}
                                        initial={{ opacity: 0, x: -10 }}
                                        animate={{ opacity: 1, x: 0 }}
                                        className="p-4 rounded-2xl bg-sentinel-blue/10 border border-sentinel-blue/20 space-y-3"
                                    >
                                        <div className="text-[10px] font-black text-sentinel-blue uppercase tracking-tight line-clamp-1">{op.target_file}</div>
                                        <div className="flex items-center gap-2">
                                            <div className="flex-1 h-1 bg-sentinel-blue/20 rounded-full overflow-hidden">
                                                <motion.div
                                                    initial={{ width: 0 }}
                                                    animate={{ width: `${op.progress_pct}%` }}
                                                    className="h-full bg-sentinel-blue shadow-[0_0_10px_rgba(0,217,255,0.5)]"
                                                />
                                            </div>
                                            <span className="text-[8px] font-mono text-sentinel-blue">{op.progress_pct ? op.progress_pct.toFixed(0) : 0}%</span>
                                        </div>
                                        <span className="text-[8px] font-mono text-sentinel-blue/60 uppercase">{op.engine || 'Veo 3 / Imagen'}</span>
                                    </motion.div>
                                ))
                            )}
                        </AnimatePresence>
                    </div>
                </div>

                {/* FASE 3: PROCESAMIENTO */}
                <div className="flex flex-col space-y-4">
                    <div className="flex items-center justify-between px-2">
                        <span className="text-[10px] font-black text-white/40 uppercase tracking-[0.2em]">3. Procesamiento</span>
                        <Settings size={14} className="text-white/20" />
                    </div>
                    <div className="flex-1 bg-white/[0.01] border border-white/5 rounded-[2.5rem] p-6 space-y-4 overflow-y-auto custom-scrollbar">
                        <AnimatePresence>
                            {processingOps.length === 0 ? (
                                <div className="h-full flex flex-col items-center justify-center opacity-10 space-y-2">
                                    <ArrowUpRight size={24} />
                                    <span className="text-[8px] font-black tracking-widest uppercase">FFmpeg Standby</span>
                                </div>
                            ) : (
                                processingOps.map((op) => (
                                    <motion.div
                                        key={op.id}
                                        initial={{ opacity: 0, scale: 0.95 }}
                                        animate={{ opacity: 1, scale: 1 }}
                                        className="p-4 rounded-2xl bg-white/[0.05] border border-white/10 space-y-2"
                                    >
                                        <div className="text-[10px] font-black text-white/60 uppercase tracking-tight line-clamp-1">{op.target_file}</div>
                                        <div className="flex justify-between items-center">
                                            <span className="text-[8px] font-mono text-white/20 uppercase">STITCHING...</span>
                                            <RefreshCw size={10} className="animate-spin text-sentinel-blue" />
                                        </div>
                                    </motion.div>
                                ))
                            )}
                        </AnimatePresence>
                    </div>
                </div>

                {/* FASE 4: CRISTALIZADO */}
                <div className="flex flex-col space-y-4">
                    <div className="flex items-center justify-between px-2">
                        <span className="text-[10px] font-black text-sentinel-green uppercase tracking-[0.2em]">4. Finalizado</span>
                        <CheckCircle size={14} className="text-sentinel-green" />
                    </div>
                    <div className="flex-1 bg-sentinel-green/[0.02] border border-sentinel-green/10 rounded-[2.5rem] p-6 space-y-4 overflow-y-auto custom-scrollbar">
                        <AnimatePresence>
                            {publishedOps.map((op, idx) => (
                                <motion.div
                                    key={idx}
                                    initial={{ opacity: 0, scale: 0.95 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    className="p-4 rounded-2xl bg-white/[0.02] border border-sentinel-green/20 group hover:bg-sentinel-green/5 transition-all"
                                >
                                    <div className="flex justify-between items-start mb-3">
                                        <FileVideo size={16} className="text-sentinel-green" />
                                        <ExternalLink size={12} className="text-white/10 group-hover:text-sentinel-green transition-colors" />
                                    </div>
                                    <div className="text-[10px] font-black text-white/80 uppercase tracking-tight line-clamp-1">{op.target_file}</div>
                                    <div className="mt-3 py-1 px-2 rounded bg-sentinel-green/10 text-sentinel-green text-[7px] font-black uppercase tracking-widest inline-block">Listo para Publicar</div>
                                </motion.div>
                            ))}
                        </AnimatePresence>
                    </div>
                </div>
            </div>

            {/* FOOTER: System Status & GPU */}
            <div className="mt-auto pt-6 border-t border-white/5 flex items-center justify-between shrink-0">
                <div className="flex items-center gap-6">
                    <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/[0.02] border border-white/5">
                        <div className={`w-1.5 h-1.5 rounded-full ${gpuStatus.includes('Active') || gpuStatus.includes('Completed') ? 'bg-sentinel-green animate-pulse' : 'bg-yellow-500'}`} />
                        <span className="text-[9px] font-black text-white/40 uppercase tracking-widest font-mono">
                            GPU ACCELERATION: <span className={gpuStatus.includes('Active') || gpuStatus.includes('Completed') ? 'text-sentinel-green' : 'text-yellow-500'}>
                                {gpuStatus.includes('Completed') ? 'NVENC ACTIVE' : 'CPU FALLBACK'}
                            </span>
                        </span>
                    </div>
                    <div className="flex items-center gap-2 opacity-30">
                        <Database size={12} />
                        <span className="text-[9px] font-mono">/dev/shm (RAM Disk) Online</span>
                    </div>
                </div>
                <div className="text-[9px] font-black text-white/20 uppercase tracking-[0.3em]">
                    Sentinel Media Core v1.3
                </div>
            </div>
        </div >
    );
};

export default FactoryView;
