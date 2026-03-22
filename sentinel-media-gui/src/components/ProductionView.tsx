import React, { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    Factory,
    FileText,
    Clock,
    CheckCircle,
    AlertCircle,
    Scissors,
    BarChart3,
    RefreshCw,
    ArrowRight,
    Film,
    FileVideo,
    Hourglass,
} from 'lucide-react';
import { motion } from 'framer-motion';

// ─── Tipos ────────────────────────────────────────────────────────────────────

interface OpEntry {
    id: string;
    status: string;
    target_file: string;
    updated_at?: string;
    op_type: string;
    engine?: string | null;
    progress_pct: number | null;
}

interface VaultFile {
    name: string;
    path: string;
    modified_at: string;
    size_bytes: number;
}

interface ProductionStats {
    total_operations: number;
    pending: number;
    running: number;
    completed: number;
    failed: number;
    videos_ready_for_stitch: number;
    avg_generation_time_mins: number;
    active_vertex_projects: string[];
}

interface MarketingAsset {
    platform: string;
    path: string;
    exists: boolean;
    content: string;
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

const PHASE_CONFIG = [
    {
        key: 'pending',
        label: 'PENDIENTE',
        icon: Hourglass,
        color: 'text-white/40',
        border: 'border-white/10',
        bg: 'bg-white/[0.02]',
        dot: 'bg-white/20',
        statuses: ['Pending', 'Queued'],
    },
    {
        key: 'running',
        label: 'GENERANDO',
        icon: RefreshCw,
        color: 'text-sentinel-blue',
        border: 'border-sentinel-blue/20',
        bg: 'bg-sentinel-blue/[0.03]',
        dot: 'bg-sentinel-blue animate-pulse',
        statuses: ['Running', 'Processing', 'Scanning'],
    },
    {
        key: 'stitch',
        label: 'LISTO PARA STITCH',
        icon: Scissors,
        color: 'text-yellow-400',
        border: 'border-yellow-400/20',
        bg: 'bg-yellow-400/[0.03]',
        dot: 'bg-yellow-400',
        statuses: ['ReadyForStitch', 'Stitching'],
    },
    {
        key: 'done',
        label: 'COMPLETADO',
        icon: CheckCircle,
        color: 'text-sentinel-green',
        border: 'border-sentinel-green/20',
        bg: 'bg-sentinel-green/[0.03]',
        dot: 'bg-sentinel-green',
        statuses: ['Done', 'Completed'],
    },
    {
        key: 'failed',
        label: 'FALLIDO',
        icon: AlertCircle,
        color: 'text-red-400',
        border: 'border-red-400/20',
        bg: 'bg-red-400/[0.03]',
        dot: 'bg-red-400',
        statuses: ['Failed', 'Error', 'Lost'],
    },
] as const;

function assignPhase(op: OpEntry): string {
    const s = op.status;
    for (const phase of PHASE_CONFIG) {
        if ((phase.statuses as readonly string[]).some(ps => s.toLowerCase().startsWith(ps.toLowerCase()))) {
            return phase.key;
        }
    }
    return 'pending';
}

function basename(path: string): string {
    return path.split('/').pop() ?? path;
}

function opIcon(op: OpEntry) {
    const type = op.op_type.toLowerCase();
    if (type.includes('stitch')) return <Film size={12} className="flex-shrink-0" />;
    if (type.includes('video')) return <FileVideo size={12} className="flex-shrink-0" />;
    if (type.includes('upload')) return <RefreshCw size={12} className="flex-shrink-0 text-sentinel-blue" />;
    return <FileText size={12} className="flex-shrink-0" />;
}

// ─── Subcomponentes ──────────────────────────────────────────────────────────

const KPITile: React.FC<{ value: number | string; label: string; sub?: string; color?: string }> = ({
    value, label, sub, color = 'text-white'
}) => (
    <div className="p-5 rounded-2xl bg-white/[0.02] border border-white/5 flex flex-col gap-1">
        <span className={`text-3xl font-black tabular-nums ${color}`}>{value}</span>
        <span className="text-[9px] font-black uppercase tracking-[0.3em] text-white/30">{label}</span>
        {sub && <span className="text-[9px] font-mono text-white/20">{sub}</span>}
    </div>
);

interface KanbanColumnProps {
    phase: typeof PHASE_CONFIG[number];
    ops: OpEntry[];
}

const KanbanColumn: React.FC<KanbanColumnProps> = ({ phase, ops }) => {
    const Icon = phase.icon;
    return (
        <div className={`flex flex-col rounded-2xl border ${phase.border} ${phase.bg} p-4 min-h-[200px]`}>
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                    <div className={`w-2 h-2 rounded-full ${phase.dot}`} />
                    <Icon size={13} className={phase.color} />
                    <span className={`text-[9px] font-black uppercase tracking-[0.3em] ${phase.color}`}>
                        {phase.label}
                    </span>
                </div>
                <span className="text-[9px] font-mono text-white/30 bg-white/5 px-2 py-0.5 rounded-full">
                    {ops.length}
                </span>
            </div>

            {/* Cards */}
            <div className="flex flex-col gap-2 flex-1 overflow-y-auto custom-scrollbar max-h-[320px]">
                {ops.length === 0 ? (
                    <div className="flex-1 flex items-center justify-center text-white/10 text-[9px] uppercase tracking-widest py-6">
                        Vacío
                    </div>
                ) : (
                    ops.map((op) => (
                        <motion.div
                            key={op.id}
                            initial={{ opacity: 0, y: 6 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="p-3 rounded-xl bg-black/20 border border-white/[0.05] group hover:bg-black/30 transition-all"
                        >
                            <div className={`flex items-center gap-1.5 mb-1 ${phase.color}`}>
                                {opIcon(op)}
                                <span className="text-[8px] font-black uppercase tracking-wider opacity-60">
                                    {op.op_type}
                                </span>
                            </div>
                            <p className="text-[10px] font-mono text-white/60 truncate" title={op.target_file}>
                                {basename(op.target_file)}
                            </p>
                            {op.progress_pct != null && op.progress_pct > 0 && (
                                <div className="mt-2 h-0.5 bg-white/5 rounded-full overflow-hidden">
                                    <div
                                        className={`h-full rounded-full ${phase.color === 'text-sentinel-blue' ? 'bg-sentinel-blue/50' : 'bg-white/20'}`}
                                        style={{ width: `${op.progress_pct}%` }}
                                    />
                                </div>
                            )}
                            {op.updated_at && (
                                <p className="text-[8px] font-mono text-white/20 mt-1 truncate">{op.updated_at}</p>
                            )}
                        </motion.div>
                    ))
                )}
            </div>
        </div>
    );
};

// ─── Vista Principal ──────────────────────────────────────────────────────────

const ProductionView: React.FC = () => {
    const [ops, setOps] = useState<OpEntry[]>([]);
    const [stats, setStats] = useState<ProductionStats | null>(null);
    const [vaultFiles, setVaultFiles] = useState<VaultFile[]>([]);
    const [marketingAssets, setMarketingAssets] = useState<MarketingAsset[]>([]);
    const [loading, setLoading] = useState(true);
    const [lastRefresh, setLastRefresh] = useState<Date>(new Date());

    const load = useCallback(async () => {
        try {
            const [rawOps, rawStats, rawVault, rawMarketing] = await Promise.all([
                invoke<OpEntry[]>('get_operaciones').catch(() => []),
                invoke<ProductionStats>('get_estadisticas_fabrica').catch(() => null),
                invoke<VaultFile[]>('get_archivos_sentinel_media').catch(() => []),
                invoke<MarketingAsset[]>('get_marketing_assets').catch(() => []),
            ]);
            setOps(rawOps);
            setStats(rawStats);
            setVaultFiles(rawVault);
            setMarketingAssets(rawMarketing);
            setLastRefresh(new Date());
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        load();
        const interval = setInterval(load, 10000);
        return () => clearInterval(interval);
    }, [load]);

    // Distribuir operaciones por fase
    const byPhase: Record<string, OpEntry[]> = Object.fromEntries(
        PHASE_CONFIG.map(p => [p.key, []])
    );
    ops.forEach(op => {
        const key = assignPhase(op);
        if (byPhase[key]) {
            byPhase[key].push(op);
        }
    });

    // Archivos sin operación activa
    const activeTargets = new Set(ops.map(o => basename(o.target_file).replace('.mp4', '.md')));
    const unprocessed = vaultFiles.filter(f => !activeTargets.has(f.name));

    const successRate = stats && stats.total_operations > 0
        ? Math.round((stats.completed / stats.total_operations) * 100)
        : 0;

    return (
        <div className="p-10 space-y-10 overflow-y-auto h-full bg-cyber-dark text-white antialiased">
            {/* HEADER */}
            <header className="flex justify-between items-start">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <Factory className="text-sentinel-blue" size={20} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase">
                            PLAN DE <span className="text-white/20">PRODUCCIÓN</span>
                        </h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase font-mono">
                        Pipeline de Contenido // Canal SecurePenguin
                    </p>
                </div>

                <div className="flex items-center gap-3">
                    <span className="text-[9px] font-mono text-white/20">
                        Actualizado {lastRefresh.toLocaleTimeString('es-ES', { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                    </span>
                    <button
                        onClick={load}
                        disabled={loading}
                        className="p-2.5 rounded-xl bg-white/[0.03] border border-white/5 hover:bg-white/[0.06] transition-all disabled:opacity-40"
                    >
                        <RefreshCw size={14} className={`text-white/40 ${loading ? 'animate-spin' : ''}`} />
                    </button>
                </div>
            </header>

            {/* KPIs */}
            {stats && (
                <section className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
                    <KPITile value={stats.total_operations} label="Total Ops" />
                    <KPITile value={stats.running} label="En Proceso" color="text-sentinel-blue" />
                    <KPITile value={stats.pending} label="En Cola" color="text-white/60" />
                    <KPITile value={stats.completed} label="Completadas" color="text-sentinel-green" />
                    <KPITile value={stats.failed} label="Fallidas" color="text-red-400" />
                    <KPITile
                        value={`${successRate}%`}
                        label="Tasa Éxito"
                        color={successRate >= 70 ? 'text-sentinel-green' : successRate >= 40 ? 'text-yellow-400' : 'text-red-400'}
                        sub={`avg ${stats.avg_generation_time_mins.toFixed(1)} min/op`}
                    />
                </section>
            )}

            {/* PIPELINE KANBAN */}
            <section>
                <div className="flex items-center gap-3 mb-6">
                    <BarChart3 size={14} className="text-sentinel-blue" />
                    <h2 className="text-[10px] font-black uppercase tracking-[0.4em] text-white/40">
                        Pipeline de Producción
                    </h2>
                    <div className="flex-1 h-px bg-white/5" />
                </div>

                {/* Flujo visual */}
                <div className="flex items-center gap-2 mb-4 text-[8px] font-black uppercase tracking-widest text-white/20">
                    {PHASE_CONFIG.map((p, i) => (
                        <React.Fragment key={p.key}>
                            <span className={i === 0 ? 'text-white/30' : ''}>{p.label}</span>
                            {i < PHASE_CONFIG.length - 1 && <ArrowRight size={10} className="text-white/10 flex-shrink-0" />}
                        </React.Fragment>
                    ))}
                </div>

                <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
                    {PHASE_CONFIG.map((phase) => (
                        <KanbanColumn
                            key={phase.key}
                            phase={phase}
                            ops={byPhase[phase.key] ?? []}
                        />
                    ))}
                </div>
            </section>

            {/* ESTRATEGIA SOCIAL (NEW) */}
            <section>
                <div className="flex items-center gap-3 mb-6">
                    <RefreshCw size={14} className="text-sentinel-green" />
                    <h2 className="text-[10px] font-black uppercase tracking-[0.4em] text-white/40">
                        Estrategia Social (Enjambre v2)
                    </h2>
                    <div className="flex-1 h-px bg-white/5" />
                </div>

                <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
                    {marketingAssets.map((asset) => (
                        <div 
                            key={asset.platform}
                            className={`p-5 rounded-[2rem] border transition-all ${asset.exists ? 'bg-sentinel-green/[0.03] border-sentinel-green/20' : 'bg-white/[0.02] border-white/5 opacity-40'}`}
                        >
                            <div className="flex justify-between items-start mb-4">
                                <span className={`text-[10px] font-black uppercase tracking-widest ${asset.exists ? 'text-sentinel-green' : 'text-white/20'}`}>
                                    {asset.platform}
                                </span>
                                {asset.exists && <CheckCircle size={10} className="text-sentinel-green" />}
                            </div>
                            <p className="text-[9px] font-mono text-white/40 mb-2 truncate">
                                {asset.exists ? basename(asset.path) : 'Pendiente...'}
                            </p>
                            <div className="h-1 bg-white/5 rounded-full overflow-hidden">
                                <div className={`h-full ${asset.exists ? 'bg-sentinel-green w-full' : 'w-0'}`} />
                            </div>
                        </div>
                    ))}
                </div>
            </section>

            {/* ARCHIVOS SIN PROCESAR */}
            {unprocessed.length > 0 && (
                <section>
                    <div className="flex items-center gap-3 mb-4">
                        <Clock size={14} className="text-yellow-400" />
                        <h2 className="text-[10px] font-black uppercase tracking-[0.4em] text-white/40">
                            Archivos Pendientes de Producir
                        </h2>
                        <span className="text-[9px] font-mono text-yellow-400/60 bg-yellow-400/10 px-2 py-0.5 rounded-full">
                            {unprocessed.length}
                        </span>
                        <div className="flex-1 h-px bg-white/5" />
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
                        {unprocessed.slice(0, 12).map((file) => (
                            <div
                                key={file.path}
                                className="group p-4 rounded-2xl bg-white/[0.02] border border-white/5 hover:bg-white/[0.04] transition-all"
                            >
                                <div className="flex items-start gap-3">
                                    <FileText size={14} className="text-white/30 flex-shrink-0 mt-0.5" />
                                    <div className="min-w-0 flex-1">
                                        <p className="text-xs font-bold text-white/70 truncate" title={file.name}>
                                            {file.name.replace('.md', '')}
                                        </p>
                                        <p className="text-[9px] font-mono text-white/20 mt-0.5">
                                            {(file.size_bytes / 1024).toFixed(1)} KB
                                        </p>
                                        <p className="text-[8px] font-mono text-white/15 mt-0.5">
                                            {new Date(file.modified_at).toLocaleDateString('es-ES')}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        ))}
                        {unprocessed.length > 12 && (
                            <div className="p-4 rounded-2xl border border-dashed border-white/5 flex items-center justify-center">
                                <span className="text-[9px] text-white/20 uppercase tracking-widest">
                                    +{unprocessed.length - 12} más
                                </span>
                            </div>
                        )}
                    </div>
                </section>
            )}
        </div>
    );
};

export default ProductionView;
