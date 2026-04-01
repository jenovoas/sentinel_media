import React, { useEffect, useState, useRef } from 'react';
import ForceGraph2D from 'react-force-graph-2d';
import { invoke } from '@tauri-apps/api/core';
import { Activity, Share2, ZoomIn, ZoomOut, RefreshCw } from 'lucide-react';

interface Node {
    id: string;
    label: string;
    group: string;
    x?: number;
    y?: number;
}

interface Link {
    source: string;
    target: string;
}

interface GraphData {
    nodes: Node[];
    links: Link[];
}

export interface EbpfEvent {
    timestamp_ns: number;
    pid: number;
    event_type: string;
    entropy_s60_raw: number;
    severity: number;
}

const VaultMap: React.FC = () => {
    const [data, setData] = useState<GraphData>({ nodes: [], links: [] });
    const [metrics, setMetrics] = useState({ coherence: 0, efficiency: 0, timestamp_s60: 0 });
    const [ebpfEvents, setEbpfEvents] = useState<EbpfEvent[]>([]);
    const [blockStats, setBlockStats] = useState({ lsm_blocks: 0, xdp_drops: 0, pulses: 0 });
    const [loading, setLoading] = useState(true);
    const [wsStatus, setWsStatus] = useState<'connecting' | 'connected' | 'error'>('connecting');
    const [error, setError] = useState<string | null>(null);
    const fgRef = useRef<any>(null);

    const fetchGraph = async () => {
        setLoading(true);
        try {
            const graph = await invoke<GraphData>('get_vault_graph');
            // Filtrar links que apuntan a nodos inexistentes (opcional pero recomendado)
            const nodeIds = new Set(graph.nodes.map(n => n.id));
            const validLinks = graph.links.filter(l => nodeIds.has(l.source) && nodeIds.has(l.target));
            
            setData({ nodes: graph.nodes, links: validLinks });
            setError(null);
        } catch (err) {
            console.error("Fallo al cargar mapa de bóveda:", err);
            setError(String(err));
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchGraph();
        
        // Conexión Telemetría Central S60
        const ws = new WebSocket('ws://127.0.0.1:8000/api/v1/telemetry');
        
        ws.onopen = () => setWsStatus('connected');
        ws.onerror = () => setWsStatus('error');
        ws.onclose = () => setWsStatus('error');
        
        ws.onmessage = (event) => {
            try {
                const liveData = JSON.parse(event.data);
                
                // Si el mensaje tiene event_type, es un eBPF Event nativo
                if (liveData.event_type) {
                    const ebpfEvent = liveData as EbpfEvent;
                    
                    // Actualizar contadores de Dropped
                    setBlockStats(prev => {
                        return {
                            ...prev,
                            lsm_blocks: ebpfEvent.event_type.includes('BLOCKED') ? prev.lsm_blocks + 1 : prev.lsm_blocks,
                            xdp_drops: ebpfEvent.event_type === 'NETWORK_BURST' ? prev.xdp_drops + 1 : prev.xdp_drops,
                            pulses: ebpfEvent.event_type === 'BIO_PULSE' ? prev.pulses + 1 : prev.pulses,
                        };
                    });

                    // Mantener un buffer de los últimos 15 eventos para el Terminal HUD
                    setEbpfEvents(prev => [ebpfEvent, ...prev].slice(0, 15));
                } else {
                    // Métricas Legacy del Canvas
                    setMetrics(prev => ({...prev, ...liveData}));
                }
            } catch (e) {
                console.error("Parseo eBPF fallido", e);
            }
        };
        
        return () => ws.close();
    }, []);

    // Colorizador Dinámico para Eventos
    const getEventColor = (type: string) => {
        if (type.includes('BLOCKED') || type === 'NETWORK_BURST') return 'text-red-500 bg-red-500/10 border-red-500/30';
        if (type.includes('ALLOWED')) return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30';
        if (type === 'BIO_PULSE') return 'text-[#00d3ff] bg-[#00d3ff]/10 border-[#00d3ff]/30';
        return 'text-yellow-400 bg-yellow-400/10 border-yellow-400/30';
    };

    if (loading && data.nodes.length === 0) {
        return (
            <div className="h-full flex flex-col items-center justify-center bg-cyber-dark/50 backdrop-blur-xl">
                <Activity className="animate-spin text-sentinel-blue mb-4" size={48} />
                <p className="text-white/40 uppercase tracking-[0.3em] text-xs font-black">Sincronizando Grafo de Conocimiento...</p>
            </div>
        );
    }

    return (
        <div className="h-full relative overflow-hidden bg-cyber-dark selection:bg-sentinel-blue/30 antialiased">
            {/* Overlay de Control */}
            <div className="absolute top-8 left-8 z-10 space-y-4">
                <div className="p-6 glass rounded-3xl border border-white/5 space-y-2">
                    <div className="flex items-center gap-3">
                        <Share2 className="text-sentinel-blue" size={20} />
                        <h2 className="text-xl font-black tracking-tighter uppercase">Mapa de <span className="text-white/20">Bóveda</span></h2>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.2em] uppercase font-mono">
                        {data.nodes.length} Entidades // {data.links.length} Conexiones
                    </p>
                </div>

                <div className="flex gap-2">
                    <button 
                        onClick={() => fgRef.current?.zoomToFit(400)}
                        className="p-3 glass rounded-xl border border-white/5 hover:bg-white/5 text-white/60 transition-all"
                        title="Ajustar Vista"
                    >
                        <RefreshCw size={16} />
                    </button>
                    <button 
                        onClick={() => fgRef.current?.zoom(fgRef.current.zoom() * 1.2)}
                        className="p-3 glass rounded-xl border border-white/5 hover:bg-white/5 text-white/60 transition-all"
                    >
                        <ZoomIn size={16} />
                    </button>
                    <button 
                        onClick={() => fgRef.current?.zoom(fgRef.current.zoom() * 0.8)}
                        className="p-3 glass rounded-xl border border-white/5 hover:bg-white/5 text-white/60 transition-all"
                    >
                        <ZoomOut size={16} />
                    </button>
                </div>
            </div>

            {/* HUD de Telemetría Ring-0 (S60) Central de Mando */}
            <div className="absolute top-8 right-8 z-10 space-y-4">
                <div className="p-6 glass rounded-2xl border border-[#00d3ff]/20 bg-[#050505]/80 backdrop-blur-xl shadow-[0_0_30px_rgba(0,211,255,0.05)] min-w-[320px]">
                    <div className="flex justify-between items-center mb-6">
                        <div className="flex items-center gap-3">
                            <div className={`w-3 h-3 rounded-full ${wsStatus === 'connected' ? 'bg-[#00d3ff] animate-pulse shadow-[0_0_15px_#00d3ff]' : 'bg-red-500'}`} />
                            <h2 className="text-xl font-black tracking-[0.2em] text-white uppercase">
                                Fénix <span className="text-[#00d3ff]">Ring-0</span>
                            </h2>
                        </div>
                        <span className="text-[10px] uppercase font-bold text-white/40 tracking-widest bg-white/5 py-1 px-3 rounded-full">En Vivo</span>
                    </div>

                    <div className="grid grid-cols-2 gap-4 mb-6">
                        <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-4">
                            <h3 className="text-[10px] text-red-500/80 uppercase font-black tracking-widest mb-1">Syst-Blocks</h3>
                            <p className="text-3xl font-mono font-black text-red-400">{blockStats.lsm_blocks}</p>
                        </div>
                        <div className="bg-[#00d3ff]/10 border border-[#00d3ff]/20 rounded-xl p-4">
                            <h3 className="text-[10px] text-[#00d3ff]/80 uppercase font-black tracking-widest mb-1">XDP Drops</h3>
                            <p className="text-3xl font-mono font-black text-[#00d3ff]">{blockStats.xdp_drops}</p>
                        </div>
                    </div>

                    <div className="space-y-3 pt-4 border-t border-white/10">
                        <div className="flex justify-between items-center">
                            <span className="text-xs font-bold text-white/50 tracking-widest uppercase flex items-center gap-2">
                                <span className="w-1.5 h-1.5 rounded-full bg-yellow-400/50"></span> Pulso Cuántico S60
                            </span>
                            <span className="font-mono text-sm text-yellow-400">{blockStats.pulses}</span>
                        </div>
                        <div className="flex justify-between items-center">
                            <span className="text-xs font-bold text-white/50 tracking-widest uppercase flex items-center gap-2">
                                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400/50"></span> Coherencia Entallada
                            </span>
                            <span className="font-mono text-sm text-emerald-400">{metrics.coherence || '0.999'}</span>
                        </div>
                    </div>
                </div>

                {/* Terminal de Eventos - La Jaula de Cristal */}
                <div className="p-4 glass rounded-2xl border border-white/10 bg-[#050505]/90 backdrop-blur-md max-h-[400px] overflow-hidden flex flex-col">
                    <div className="flex items-center gap-2 mb-4 pb-3 border-b border-white/5">
                        <Activity size={14} className="text-[#00d3ff]" />
                        <h3 className="text-xs font-bold uppercase tracking-[0.2em] text-white/70">Terminal eBPF Intercepts</h3>
                    </div>
                    
                    <div className="flex-1 overflow-y-auto pr-2 space-y-2 custom-scrollbar">
                        {ebpfEvents.length === 0 ? (
                            <p className="text-xs text-white/20 font-mono italic text-center py-8">Esperando colisiones Ring-0...</p>
                        ) : (
                            ebpfEvents.map((evt, idx) => (
                                <div key={idx} className={`p-3 rounded-lg border flex flex-col gap-1 transition-all animate-in fade-in slide-in-from-right-4 ${getEventColor(evt.event_type)}`}>
                                    <div className="flex justify-between items-center">
                                        <span className="text-[10px] font-black tracking-widest uppercase">{evt.event_type}</span>
                                        <span className="text-[10px] font-mono opacity-60">PID: {evt.pid}</span>
                                    </div>
                                    <div className="flex justify-between items-end">
                                        <span className="font-mono text-xs font-bold tracking-tight opacity-90">ENTROPY: {evt.entropy_s60_raw}</span>
                                        <span className="text-[9px] uppercase font-bold opacity-50">S60</span>
                                    </div>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>

            {error && (
                <div className="absolute bottom-8 left-8 z-10 p-4 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-[10px] font-black uppercase tracking-widest">
                    Error: {error}
                </div>
            )}

            <ForceGraph2D
                ref={fgRef}
                graphData={data}
                backgroundColor="#050505"
                nodeLabel="label"
                nodeAutoColorBy="group"
                linkDirectionalParticles={2}
                linkDirectionalParticleSpeed={0.005}
                linkDirectionalParticleWidth={1.5}
                linkDirectionalParticleColor={() => "#00d3ff"}
                linkColor={() => "#ffffff10"}
                nodeCanvasObject={(node: any, ctx, globalScale) => {
                    const label = node.label;
                    const fontSize = 12 / globalScale;
                    ctx.font = `${fontSize}px Inter, sans-serif`;
                    
                    // Glow effect
                    ctx.shadowBlur = 15;
                    ctx.shadowColor = "#00d3ff";
                    
                    // Node circle
                    ctx.fillStyle = node.color || "#00d3ff";
                    ctx.beginPath();
                    ctx.arc(node.x, node.y, 4, 0, 2 * Math.PI, false);
                    ctx.fill();
                    
                    // Label
                    if (globalScale > 3) {
                        ctx.shadowBlur = 0;
                        ctx.fillStyle = "rgba(255, 255, 255, 0.6)";
                        ctx.textAlign = "center";
                        ctx.textBaseline = "middle";
                        ctx.fillText(label, node.x, node.y + 8);
                    }
                }}
            />

            {/* Ruido de fondo */}
            <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-[0.03] pointer-events-none" />
        </div>
    );
};

export default VaultMap;
