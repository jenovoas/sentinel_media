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

const VaultMap: React.FC = () => {
    const [data, setData] = useState<GraphData>({ nodes: [], links: [] });
    const [loading, setLoading] = useState(true);
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
    }, []);

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
