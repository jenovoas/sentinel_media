import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
    Search,
    FileText,
    Brain,
    ChevronRight,
    TrendingUp,
    BookOpen,
    Video
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface Report {
    title: string;
    path: string;
    content_preview: string;
}

interface ResearchAgentStatus {
    state: string;
    rag_loaded: boolean;
    rag_doc_count: number;
    web_search_ready: boolean;
    translators_active: boolean;
}



// Componente contenedor
const RESEARCH_MODES = ['ARCHITECT', 'LINGUIST', 'PATTERN', 'GENEALOGIST', 'DEEP_DIVE'] as const;
type ResearchMode = typeof RESEARCH_MODES[number];

const ResearchView: React.FC = () => {
    const [reports, setReports] = useState<Report[]>([]);
    const [activeTasks, setActiveTasks] = useState<string[]>([]);
    const [selectedMode, setSelectedMode] = useState<ResearchMode>('DEEP_DIVE');
    const [groundingEnabled, setGroundingEnabled] = useState(true);
    const [query, setQuery] = useState('');
    const [isRunning, setIsRunning] = useState(false);
    const [feedback, setFeedback] = useState<{ type: 'ok' | 'error'; msg: string } | null>(null);
    const [agentStatus, setAgentStatus] = useState<ResearchAgentStatus>({
        state: 'OFFLINE',
        rag_loaded: false,
        rag_doc_count: 0,
        web_search_ready: false,
        translators_active: false
    });

    const fetchData = async () => {
        try {
            const res = await invoke<Report[]>('get_reportes_investigacion');
            const status = await invoke<ResearchAgentStatus>('get_estado_agente_investigacion');
            const tasks = await invoke<string[]>('get_tareas_investigacion_activas');
            setReports(res);
            setAgentStatus(status);
            setActiveTasks(tasks);
        } catch (e) {
            console.error(e);
        }
    };

    useEffect(() => {
        fetchData();
        const interval = setInterval(fetchData, 10000); // Sondeo reducido a 10s como respaldo

        let unlistenTaskCompleted: UnlistenFn;
        let unlistenReportsUpdated: UnlistenFn;

        const setupListeners = async () => {
            unlistenTaskCompleted = await listen('tarea-investigacion-completada', (event: any) => {
                console.log('Evento: Tarea de investigacion completada', event.payload);
                fetchData(); // Refresco inmediato
                // Aqui se podria agregar una notificacion tipo toast
            });

            unlistenReportsUpdated = await listen('reportes-investigacion-actualizados', () => {
                console.log('Evento: Reportes de investigacion actualizados');
                fetchData();
            });
        };

        setupListeners();

        return () => {
            clearInterval(interval);
            if (unlistenTaskCompleted) unlistenTaskCompleted();
            if (unlistenReportsUpdated) unlistenReportsUpdated();
        };
    }, []);

    const handleStartResearch = async () => {
        if (query.trim().length === 0 || isRunning) return;
        setIsRunning(true);
        setFeedback(null);
        try {
            await invoke('iniciar_tarea_investigacion', { query, mode: selectedMode, grounding: groundingEnabled });
            setQuery('');
            setFeedback({ type: 'ok', msg: 'Protocolo iniciado. Revisa los reportes en breve.' });
            setTimeout(() => setFeedback(null), 4000);
        } catch (e) {
            console.error(e);
            setFeedback({ type: 'error', msg: `Error: ${e}` });
        } finally {
            setIsRunning(false);
        }
    };

    return (
        <div className="p-10 space-y-10 h-full overflow-hidden flex flex-col bg-cyber-dark text-white selection:bg-sentinel-blue/30 antialiased font-sans">
            <header className="flex justify-between items-end">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <Search className="text-sentinel-blue" size={20} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase">GESTIÓN DE <span className="text-white/20">INVESTIGACIÓN</span></h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase font-mono">Síntesis de Conocimiento // Deep Research Node</p>
                </div>

                {activeTasks.length > 0 && (
                    <div className="flex items-center gap-4 animate-pulse">
                        <div className="flex items-center gap-2 px-4 py-2 rounded-xl bg-sentinel-green/10 border border-sentinel-green/20 text-[10px] font-black uppercase tracking-widest text-sentinel-green">
                            <div className="w-2 h-2 rounded-full bg-sentinel-green animate-ping" />
                            PROCESANDO TAREA: {activeTasks[0].substring(0, 30)}...
                        </div>
                    </div>
                )}
            </header>

            {/* ACTIVE PROTOCOL CONSOLE */}
            <div className="p-6 rounded-[2.5rem] bg-white/[0.02] border border-white/5 relative overflow-hidden">
                <div className="absolute top-0 right-0 p-4 opacity-10">
                    <TrendingUp size={100} />
                </div>

                <h2 className="text-xs font-black text-white/60 uppercase tracking-widest mb-6 flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-sentinel-blue animate-pulse" />
                    Protocolo de Investigación Activa
                </h2>

                <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
                    <div className="md:col-span-3 space-y-4">
                        <div className="relative group">
                            <input
                                type="text"
                                placeholder="Escribe tu hipótesis, tema o pregunta de investigación..."
                                className="w-full bg-black/20 border border-white/10 rounded-2xl px-6 py-4 text-sm font-medium text-white placeholder-white/20 focus:border-sentinel-blue/50 focus:bg-black/40 outline-none transition-all pr-12"
                                onKeyDown={(e) => { if (e.key === 'Enter') handleStartResearch(); }}
                                value={query}
                                onChange={(e) => setQuery(e.target.value)}
                                disabled={isRunning}
                            />
                            <Search className="absolute right-6 top-4.5 text-white/10 group-focus-within:text-sentinel-blue transition-colors" size={20} />
                        </div>

                        <div className="flex gap-2">
                            {RESEARCH_MODES.map((m) => (
                                <button
                                    key={m}
                                    onClick={() => setSelectedMode(m)}
                                    className={`px-4 py-2 rounded-xl border text-[9px] font-black uppercase tracking-widest transition-all hover:border-white/20 flex-1 ${selectedMode === m ? 'bg-sentinel-blue text-cyber-dark border-sentinel-blue' : 'bg-white/5 text-white/40 border-white/5'}`}
                                >
                                    {m.replace('_', ' ')}
                                </button>
                            ))}
                        </div>
                    </div>

                    <div className="flex flex-col justify-between space-y-4">
                        <div
                            className="flex items-center justify-between p-3 rounded-xl bg-white/5 border border-white/5 cursor-pointer hover:border-white/10"
                            onClick={() => setGroundingEnabled(g => !g)}
                        >
                            <span className="text-[9px] font-bold text-white/50 uppercase">Grounding (Vault)</span>
                            <div className={groundingEnabled ? 'text-sentinel-green' : 'text-white/20'}><Brain size={16} /></div>
                        </div>

                        {feedback && (
                            <p className={`text-[9px] font-mono px-3 py-2 rounded-lg ${feedback.type === 'ok' ? 'bg-sentinel-green/10 text-sentinel-green' : 'bg-red-500/10 text-red-400'}`}>
                                {feedback.msg}
                            </p>
                        )}

                        <button
                            onClick={handleStartResearch}
                            disabled={isRunning || query.trim().length === 0}
                            className="w-full py-4 rounded-xl bg-gradient-to-r from-sentinel-blue to-cyan-400 text-cyber-dark text-xs font-black uppercase tracking-widest hover:shadow-[0_0_20px_rgba(0,217,255,0.4)] transition-all flex items-center justify-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed"
                        >
                            <TrendingUp size={16} /> {isRunning ? 'Iniciando...' : 'Iniciar Análisis'}
                        </button>
                    </div>
                </div>
            </div>

            <div className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-8 overflow-hidden">
                {/* COLUMNA REPORTES */}
                <div className="lg:col-span-8 flex flex-col space-y-6 overflow-hidden">
                    <div className="flex items-center justify-between px-2">
                        <h3 className="text-xs font-black text-white/40 uppercase tracking-[0.2em] flex items-center gap-3">
                            <BookOpen size={16} /> REPORTES COMPILADOS
                        </h3>
                    </div>

                    <div className="flex-1 overflow-y-auto pr-4 space-y-4 custom-scrollbar">
                        <AnimatePresence>
                            {reports.length === 0 ? (
                                <div className="h-full flex flex-col items-center justify-center opacity-10 space-y-4">
                                    <FileText size={48} />
                                    <span className="text-xs font-black tracking-[0.2em] uppercase text-center">No se han generado reportes<br />de investigación aún</span>
                                </div>
                            ) : (
                                reports.map((report, idx) => (
                                    <motion.div
                                        key={idx}
                                        initial={{ opacity: 0, y: 20 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        className="p-8 rounded-[2.5rem] bg-white/[0.01] border border-white/5 group hover:border-sentinel-blue/20 transition-all cursor-pointer relative overflow-hidden"
                                    >
                                        <div className="flex justify-between items-start mb-6">
                                            <div className="p-3 rounded-2xl bg-sentinel-blue/5 text-sentinel-blue">
                                                <FileText size={20} />
                                            </div>
                                            <div className="flex items-center gap-2 text-[8px] font-black text-white/10 uppercase tracking-widest">
                                                {report.path.includes('translation') ? 'TRADUCCIÓN' : 'SÍNTESIS'} // Markdown
                                            </div>
                                        </div>

                                        <h4 className="text-sm font-black text-white group-hover:text-sentinel-blue transition-colors uppercase tracking-tight mb-3">
                                            {report.title.replace('_research.md', '').replace('.md', '').replace(/_/g, ' ')}
                                        </h4>

                                        <p className="text-[11px] text-white/20 leading-relaxed line-clamp-2 italic">
                                            {report.content_preview}...
                                        </p>

                                        <div className="mt-6 flex items-center justify-between opacity-0 group-hover:opacity-100 transition-all">
                                            <div className="flex items-center gap-2 text-[9px] font-black text-sentinel-blue">
                                                LEER REPORTE COMPLETO <ChevronRight size={12} />
                                            </div>

                                            <button
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    const config = {
                                                        shorts: true,
                                                        longform: false,
                                                        stitch: true,
                                                        publish: false,
                                                        local: true,
                                                        provider: 'gemini',
                                                        cinematic: false,
                                                        gpu: true,
                                                        specific_file: report.path
                                                    };
                                                    invoke('ejecutar_generacion_fabrica', { config });
                                                }}
                                                className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-sentinel-blue/20 border border-sentinel-blue/30 text-[8px] font-black text-sentinel-blue uppercase hover:bg-sentinel-blue hover:text-cyber-dark transition-all"
                                            >
                                                <Video size={12} /> PRODUCIR VIDEO
                                            </button>
                                        </div>
                                    </motion.div>
                                ))
                            )}
                        </AnimatePresence>
                    </div>
                </div>

                {/* COLUMNA STATUS AGENTE */}
                <div className="lg:col-span-4 space-y-6">
                    <div className="p-8 rounded-[2.5rem] glass-panel border border-white/5 space-y-8">
                        <div className="flex items-center justify-between">
                            <h3 className="text-xs font-black text-white/40 uppercase tracking-widest flex items-center gap-3">
                                <Brain size={16} /> ESTADO PAI
                            </h3>
                            <div className={`w-2 h-2 rounded-full ${agentStatus.state === 'RUNNING' ? 'bg-sentinel-green animate-pulse' : agentStatus.state === 'IDLE' ? 'bg-sentinel-blue' : 'bg-white/20'}`} />
                        </div>

                        <div className="space-y-6">
                            <div className="p-6 rounded-2xl bg-white/5 border border-white/5 text-center">
                                <Brain size={24} className="text-white/20 mx-auto mb-4" />
                                <div className="text-[10px] font-black text-white/20 uppercase tracking-widest mb-1">Estado de Nodo</div>
                                <div className={`text-xl font-black font-mono ${agentStatus.state === 'RUNNING' ? 'text-sentinel-green' : agentStatus.state === 'IDLE' ? 'text-sentinel-blue' : 'text-white/40'}`}>
                                    {agentStatus.state}
                                </div>
                            </div>

                            <div className="space-y-4">
                                <div className="flex justify-between text-[10px] font-black text-white/20 uppercase tracking-widest">
                                    <span>Contexto RAG ({agentStatus.rag_doc_count} docs)</span>
                                    <span className={agentStatus.rag_loaded ? 'text-sentinel-green' : 'text-red-400'}>
                                        {agentStatus.rag_loaded ? 'LOADED' : 'EMPTY'}
                                    </span>
                                </div>
                                <div className="flex justify-between text-[10px] font-black text-white/20 uppercase tracking-widest">
                                    <span>Web Search</span>
                                    <span className={agentStatus.web_search_ready ? 'text-sentinel-green' : 'text-red-400'}>
                                        {agentStatus.web_search_ready ? 'READY' : 'NO API'}
                                    </span>
                                </div>
                                <div className="flex justify-between text-[10px] font-black text-white/20 uppercase tracking-widest">
                                    <span>Translators</span>
                                    <span className={agentStatus.translators_active ? 'text-sentinel-blue' : 'text-white/20'}>
                                        {agentStatus.translators_active ? 'ACTIVE' : 'IDLE'}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className="p-8 rounded-[2.5rem] bg-gradient-to-br from-sentinel-blue/10 to-transparent border border-sentinel-blue/20 flex flex-col justify-between h-56">
                        <TrendingUp size={24} className="text-sentinel-blue" />
                        <div>
                            <h4 className="text-xs font-black text-white uppercase tracking-widest mb-2">ANÁLISIS PREDICTIVO</h4>
                            <p className="text-[10px] text-white/40 leading-relaxed uppercase tracking-tighter">
                                El agente está listo para sintetizar documentación técnica basándose en el historial de la bóveda.
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default ResearchView;
