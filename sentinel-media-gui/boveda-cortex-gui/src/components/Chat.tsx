import React, { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    Send,
    User,
    Bot,
    Terminal,
    Cpu,
    RefreshCw,
    Save,
    FolderOpen,
    Trash2,
    Plus,
    Settings,
    Shield,
    Activity,
    Key,
    Database,
    FileDown
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface Message {
    role: 'user' | 'assistant';
    content: string;
    timestamp: string;
}

interface Sentinomica {
    requests: number;
    errors: number;
    last_status: string;
    last_used: string;
}

interface UsoGlobalSentinomica {
    provider_stats: Record<string, Sentinomica>;
}

interface BalancerStatus {
    rag_index_path: string;
    rag_doc_count: number;
    gpu_info: string;
    cpu_usage_pct: number;
    memory_info: string;
    llm_cli_active: boolean;
}

const Chat: React.FC = () => {
    const [messages, setMessages] = useState<Message[]>([]);
    const [selectedModel, setSelectedModel] = useState("");
    const [availableModels, setAvailableModels] = useState<string[]>([]);
    const [input, setInput] = useState('');
    const [loading, setLoading] = useState(false);

    // Configuracion y uso
    const [showSettings, setShowSettings] = useState(false);
    const [configKeys, setConfigKeys] = useState<any>({});
    const [usageStats, setUsageStats] = useState<UsoGlobalSentinomica | null>(null);
    const [activeTab, setActiveTab] = useState<'keys' | 'traffic'>('keys');

    // Gestion de conversaciones
    const [conversations, setConversations] = useState<string[]>([]);
    const [showSaveDialog, setShowSaveDialog] = useState(false);
    const [showLoadDialog, setShowLoadDialog] = useState(false);
    const [saveName, setSaveName] = useState('');
    const [currentConversation, setCurrentConversation] = useState<string | null>(null);

    // Estado de proceso
    const [processingFile, setProcessingFile] = useState(false);
    const [systemStatus, setSystemStatus] = useState<BalancerStatus>({
        rag_index_path: 'No cargado',
        rag_doc_count: 0,
        gpu_info: 'Buscando...',
        cpu_usage_pct: 0,
        memory_info: '---',
        llm_cli_active: false
    });

    const scrollRef = useRef<HTMLDivElement>(null);

    const updateStatus = async () => {
        try {
            const status = await invoke<BalancerStatus>('get_estado_balanceador');
            setSystemStatus(status);
        } catch (e) {
            console.error('Error obteniendo estado del sistema:', e);
        }
    };

    const loadModels = async () => {
        try {
            const models = await invoke<string[]>('obtener_modelos_disponibles');
            setAvailableModels(models);
            if (models.length > 0 && (!selectedModel || !models.includes(selectedModel))) {
                setSelectedModel(models[0]);
            }
        } catch (e) {
            console.error('Error cargando modelos del cluster:', e);
            setAvailableModels([]);
        }
    };

    const loadConversationsList = async () => {
        try {
            const list = await invoke<string[]>('listar_conversaciones');
            setConversations(list);
        } catch (e) {
            console.error('Error cargando lista de conversaciones:', e);
        }
    };

    useEffect(() => {
        loadModels();
        loadConversationsList();
        updateStatus();
        const interval = setInterval(updateStatus, 5000);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [messages]);

    useEffect(() => {
        if (showSettings) {
            invoke('get_llaves_api').then((data: any) => setConfigKeys(data));
            invoke('obtener_uso_sentinomica').then((data: any) => setUsageStats(data));
        }
    }, [showSettings]);

    const handleSend = async () => {
        if (!input.trim() || loading) return;

        const userMsg: Message = {
            role: 'user',
            content: input,
            timestamp: new Date().toISOString()
        };

        const updatedMessages = [...messages, userMsg];
        setMessages(updatedMessages);
        setInput('');
        setLoading(true);

        try {
            const response = await invoke<string>('send_neural_message', { messages: updatedMessages, modelId: selectedModel });
            const assistantMsg: Message = {
                role: 'assistant',
                content: response,
                timestamp: new Date().toISOString()
            };
            setMessages(prev => [...prev, assistantMsg]);
        } catch (e) {
            const errorMsg: Message = {
                role: 'assistant',
                content: `❌ ERROR CRÍTICO (Traza del sistema):\n${e}`,
                timestamp: new Date().toISOString()
            };
            setMessages(prev => [...prev, errorMsg]);
        } finally {
            setLoading(false);
        }
    };

    const handleSaveConversation = async () => {
        if (!saveName.trim()) return;
        try {
            await invoke('guardar_conversacion', { name: saveName, messages });
            setCurrentConversation(saveName);
            setShowSaveDialog(false);
            setSaveName('');
            await loadConversationsList();
        } catch (e) {
            console.error('Error guardando conversación:', e);
        }
    };

    const handleExportConversation = async () => {
        if (!currentConversation) return alert('No hay conversación activa para exportar');
        try {
            const path = await invoke<string>('exportar_conversacion_md', { name: currentConversation });
            alert(`Conversación exportada exitosamente a: ${path}`);
        } catch (e) {
            console.error('Error exportando conversación:', e);
            alert('Error exportando conversación: ' + e);
        }
    };

    const handleLoadConversation = async (name: string) => {
        try {
            const loadedMessages = await invoke<Message[]>('cargar_conversacion', { name });
            setMessages(loadedMessages);
            setCurrentConversation(name);
            setShowLoadDialog(false);
        } catch (e) {
            console.error('Error cargando conversación:', e);
        }
    };

    const handleDeleteConversation = async (name: string) => {
        if (!confirm(`¿Eliminar conversación "${name}"?`)) return;
        try {
            await invoke('eliminar_conversacion', { name });
            if (currentConversation === name) {
                setMessages([]);
                setCurrentConversation(null);
            }
            await loadConversationsList();
        } catch (e) {
            console.error('Error eliminando conversación:', e);
        }
    };

    const handleNewConversation = () => {
        if (messages.length > 0 && !confirm('¿Iniciar nueva conversación?')) return;
        setMessages([]);
        setCurrentConversation(null);
    };

    const handleIngestMemory = async () => {
        try {
            const { open } = await import('@tauri-apps/plugin-dialog');
            const selected = await open({
                directory: true,
                multiple: false,
            });

            if (selected && typeof selected === 'string') {
                setProcessingFile(true);
                const assistantMsg: Message = {
                    role: 'assistant',
                    content: `🧠 Iniciando ingestión de memoria neuronal en: ${selected}\nProcesando...`,
                    timestamp: new Date().toISOString()
                };
                setMessages(prev => [...prev, assistantMsg]);

                try {
                    const result = await invoke<string>('ingestar_memoria', { path: selected });
                    const successMsg: Message = {
                        role: 'assistant',
                        content: `✅ INGESTIÓN COMPLETADA (Output Real):\n\n${result}`,
                        timestamp: new Date().toISOString()
                    };
                    setMessages(prev => [...prev, successMsg]);
                } catch (e) {
                    const errorMsg: Message = {
                        role: 'assistant',
                        content: `❌ FALLO EN INGESTIÓN (Error Real):\n${e}`,
                        timestamp: new Date().toISOString()
                    };
                    setMessages(prev => [...prev, errorMsg]);
                } finally {
                    setProcessingFile(false);
                }
            }
        } catch (e) {
            console.error('Error en selector de carpeta:', e);
        }
    };

    const saveSettings = async () => {
        try {
            await invoke('guardar_llaves_api', { keys: configKeys });
            setShowSettings(false);
            await loadModels();
        } catch (e) {
            alert(`Error guardando configuración: ${e}`);
        }
    };

    const formatTimestamp = (isoString: string) => {
        const date = new Date(isoString);
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };

    return (
        <div className="flex flex-col h-full overflow-hidden p-8">
            <header className="flex justify-between items-center bg-white/[0.02] p-8 rounded-[2.5rem] border border-white/5 mb-8">
                <div>
                    <div className="flex items-center gap-3 mb-2">
                        <Terminal size={24} className="text-sentinel-blue" />
                        <h1 className="text-3xl font-black text-white uppercase tracking-tighter">
                            CONSULTA <span className="text-white/40">NEURONAL (GEMINI)</span>
                        </h1>
                    </div>
                    <p className="text-white/30 text-[10px] font-bold uppercase tracking-[0.4em] font-mono">
                        {currentConversation ? `MODO: HISTORIAL (${currentConversation}.JSON)` : 'MODO: FLUJO DIRECTO'}
                    </p>
                </div>
                <div className="flex gap-3">
                    <button onClick={handleNewConversation} className="flex items-center gap-2 px-4 py-2 rounded-2xl bg-white/5 border border-white/5 text-[10px] font-black uppercase tracking-widest text-white/60 hover:text-white hover:bg-white/10 transition-all">
                        <Plus size={14} /> Nueva
                    </button>
                    <button onClick={handleExportConversation} disabled={!currentConversation} className="flex items-center gap-2 px-4 py-2 rounded-2xl bg-white/5 border border-white/5 text-[10px] font-black uppercase tracking-widest text-sentinel-blue hover:bg-sentinel-blue/20 transition-all disabled:opacity-30">
                        <FileDown size={14} /> Exportar
                    </button>
                    <button onClick={() => setShowSaveDialog(true)} disabled={messages.length === 0} className="flex items-center gap-2 px-4 py-2 rounded-2xl bg-white/5 border border-white/5 text-[10px] font-black uppercase tracking-widest text-sentinel-green hover:bg-sentinel-green/20 transition-all disabled:opacity-30">
                        <Save size={14} /> Guardar
                    </button>
                    <button onClick={() => setShowLoadDialog(true)} className="flex items-center gap-2 px-4 py-2 rounded-2xl bg-white/5 border border-white/5 text-[10px] font-black uppercase tracking-widest text-sentinel-blue hover:bg-sentinel-blue/20 transition-all">
                        <FolderOpen size={14} /> Cargar
                    </button>
                    <button onClick={handleIngestMemory} disabled={processingFile} className="flex items-center gap-2 px-4 py-2 rounded-2xl bg-sentinel-blue/5 border border-sentinel-blue/20 text-[10px] font-black uppercase tracking-widest text-sentinel-blue hover:bg-sentinel-blue hover:text-cyber-dark transition-all">
                        <Database size={14} /> ÍNDICE RAG: {systemStatus.rag_doc_count} DOCS
                    </button>
                    <button
                        onClick={async () => {
                            const query = prompt('Query para memoria:');
                            if (query) {
                                try {
                                    const result = await invoke<string>('consultar_memoria', { query });
                                    const msg: Message = { role: 'assistant', content: `🧠 MEMORIA: ${result}`, timestamp: new Date().toISOString() };
                                    setMessages(prev => [...prev, msg]);
                                } catch (e) {
                                    console.error(e);
                                }
                            }
                        }}
                        className="flex items-center gap-2 px-4 py-2 rounded-2xl bg-purple-500/5 border border-purple-500/20 text-[10px] font-black uppercase tracking-widest text-purple-400 hover:bg-purple-500 hover:text-cyber-dark transition-all"
                    >
                        <Database size={14} /> QUERY MEMORIA
                    </button>
                    <div className="flex items-center gap-2 px-4 py-2 rounded-2xl bg-white/5 border border-white/5 text-[10px] font-black uppercase tracking-widest text-sentinel-green">\n                        <Shield size={14} /> CLUSTER: {availableModels.length} NODOS\n                    </div>
                </div>
            </header>

            {/* Diálogos */}
            {showSaveDialog && (
                <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center">
                    <motion.div initial={{ opacity: 0, scale: 0.9 }} animate={{ opacity: 1, scale: 1 }} className="bg-cyber-dark border border-white/10 rounded-[2rem] p-8 max-w-md w-full">
                        <h2 className="text-xl font-black text-white mb-4 uppercase">Guardar Conversación</h2>
                        <input type="text" value={saveName} onChange={(e) => setSaveName(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && handleSaveConversation()} placeholder="Nombre de archivo..." className="w-full bg-white/5 border border-white/10 rounded-2xl px-4 py-3 text-sm text-white mb-4" autoFocus />
                        <div className="flex gap-3">
                            <button onClick={handleSaveConversation} className="flex-1 bg-sentinel-green text-cyber-dark font-black uppercase text-xs py-3 rounded-2xl">Confirmar</button>
                            <button onClick={() => setShowSaveDialog(false)} className="flex-1 bg-white/5 text-white/60 font-black uppercase text-xs py-3 rounded-2xl">Cancelar</button>
                        </div>
                    </motion.div>
                </div>
            )}

            {showLoadDialog && (
                <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center">
                    <motion.div initial={{ opacity: 0, scale: 0.9 }} animate={{ opacity: 1, scale: 1 }} className="bg-cyber-dark border border-white/10 rounded-[2rem] p-8 max-w-md w-full max-h-[80vh] flex flex-col">
                        <h2 className="text-xl font-black text-white mb-4 uppercase">Cargar Conversación</h2>
                        <div className="flex-1 overflow-y-auto space-y-2 mb-4">
                            {conversations.map((name) => (
                                <div key={name} className="flex items-center justify-between bg-white/5 border border-white/10 rounded-2xl p-4 group">
                                    <button onClick={() => handleLoadConversation(name)} className="flex-1 text-left text-sm text-white">{name}</button>
                                    <button onClick={() => handleDeleteConversation(name)} className="opacity-0 group-hover:opacity-100 text-red-400"><Trash2 size={16} /></button>
                                </div>
                            ))}
                        </div>
                        <button onClick={() => setShowLoadDialog(false)} className="w-full bg-white/5 text-white/60 font-black uppercase text-xs py-3 rounded-2xl">Cerrar</button>
                    </motion.div>
                </div>
            )}

            {/* Area de chat */}
            <div className="flex-1 overflow-hidden relative glass-panel rounded-[2.5rem] border border-white/5 flex flex-col mb-8">
                <div ref={scrollRef} className="flex-1 overflow-y-auto p-10 space-y-8 scroll-smooth">
                    {messages.length === 0 && (
                        <div className="h-full flex flex-col items-center justify-center text-center opacity-30">
                            <Bot size={64} className="mb-6" />
                            <p className="text-sm font-black uppercase tracking-widest text-white">Nodo Cortex Listo</p>
                        </div>
                    )}
                    <AnimatePresence>
                        {messages.map((msg, idx) => (
                            <motion.div key={idx} initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                                <div className={`flex gap-4 max-w-[80%] ${msg.role === 'user' ? 'flex-row-reverse' : ''}`}>
                                    <div className={`w-10 h-10 rounded-2xl flex items-center justify-center flex-shrink-0 ${msg.role === 'user' ? 'bg-white/10 text-white' : 'bg-sentinel-blue/20 text-sentinel-blue'}`}>
                                        {msg.role === 'user' ? <User size={20} /> : <Terminal size={20} />}
                                    </div>
                                    <div className="space-y-2">
                                        <div className={`p-6 rounded-[2rem] text-sm leading-relaxed whitespace-pre-wrap ${msg.role === 'user' ? 'bg-white/5 text-white/90 rounded-tr-none' : 'bg-sentinel-blue/[0.03] text-white/80 border border-white/5 rounded-tl-none font-medium'}`}>
                                            {msg.content}
                                        </div>
                                        <div className={`text-[9px] font-black uppercase tracking-widest text-white/10 px-2 ${msg.role === 'user' ? 'text-right' : 'text-left'}`}>
                                            {formatTimestamp(msg.timestamp)} // {msg.role === 'user' ? 'OPERADOR' : 'IA'}
                                        </div>
                                    </div>
                                </div>
                            </motion.div>
                        ))}
                    </AnimatePresence>
                    {loading && (
                        <div className="flex gap-4 items-center">
                            <RefreshCw size={20} className="animate-spin text-sentinel-blue" />
                            <span className="text-[10px] font-black uppercase tracking-widest text-sentinel-blue">Sintetizando respuesta...</span>
                        </div>
                    )}
                </div>
            </div>

            {/* Area de entrada */}
            <div className="relative group">
                <div className="absolute -inset-1 bg-gradient-to-r from-sentinel-blue/20 to-sentinel-green/20 rounded-[2.5rem] blur opacity-50 transition duration-1000 pointer-events-none" />
                <div className="relative flex items-center bg-white/[0.03] border border-white/10 rounded-[2.5rem] p-3 backdrop-blur-xl">
                    <input type="text" value={input} onChange={(e) => setInput(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && handleSend()} placeholder="Escriba su consulta al Cortex..." className="flex-1 bg-transparent border-none outline-none px-6 text-sm text-white" />
                    <button onClick={handleSend} disabled={loading || !input.trim()} className="w-14 h-14 rounded-full bg-sentinel-blue text-cyber-dark flex items-center justify-center hover:scale-105 transition-all disabled:opacity-50">
                        <Send size={20} />
                    </button>
                </div>
                <div className="mt-4 flex justify-between px-8 text-[9px] font-black uppercase tracking-[0.2em] font-mono relative z-10">
                    <div className="flex gap-6 text-white/30">
                        <span className="flex items-center gap-1">
                            <div className="w-1.5 h-1.5 rounded-full bg-sentinel-blue" />
                            RUTA RAG: {systemStatus.rag_index_path}
                        </span>
                        <span className="flex items-center gap-1">
                            <Cpu size={12} className="text-sentinel-green" /> HARDWARE: {systemStatus.gpu_info}
                        </span>
                        <span className="flex items-center gap-1">
                            <Activity size={12} className="text-white/40" /> SYS: CPU {systemStatus.cpu_usage_pct.toFixed(1)}% | RAM {systemStatus.memory_info}
                        </span>
                    </div>
                    <div className="flex items-center gap-3">
                        <span className="text-white/40">MODELO:</span>
                        <select value={selectedModel} onChange={(e) => setSelectedModel(e.target.value)} className="bg-black/40 border border-sentinel-blue/20 text-sentinel-blue rounded px-2 py-1 uppercase outline-none cursor-pointer hover:border-sentinel-blue/40 transition-colors">
                            {availableModels.map(m => <option key={m} value={m}>{m}</option>)}
                        </select>
                        <button onClick={loadModels} title="Recargar Modelos" className="text-white/20 hover:text-white transition-colors"><RefreshCw size={12} /></button>
                        <button onClick={() => setShowSettings(true)} title="Configuración de Tokens" className="text-white/20 hover:text-white transition-colors"><Settings size={12} /></button>
                    </div>
                </div>
            </div>

            {/* Modal de Configuración */}
            {showSettings && (
                <div className="fixed inset-0 bg-black/90 backdrop-blur-xl z-[100] flex items-center justify-center p-8 font-mono">
                    <motion.div initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} className="bg-[#0a0f18] border border-sentinel-blue/20 rounded-xl w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden shadow-2xl">
                        <div className="h-14 border-b border-white/5 flex justify-between items-center px-6 bg-black/20">
                            <span className="text-sentinel-blue font-black uppercase tracking-widest text-xs flex items-center gap-2"><Shield size={16} /> Admin Cortex (Balanceador de Tokens)</span>
                            <div className="flex gap-2">
                                <button onClick={() => setActiveTab('keys')} className={`px-4 py-1 rounded-lg text-[10px] uppercase font-bold transition-all ${activeTab === 'keys' ? 'bg-sentinel-blue/20 text-white' : 'text-white/40 hover:text-white/60'}`}>LLAVES</button>
                                <button onClick={() => setActiveTab('traffic')} className={`px-4 py-1 rounded-lg text-[10px] uppercase font-bold transition-all ${activeTab === 'traffic' ? 'bg-sentinel-blue/20 text-white' : 'text-white/40 hover:text-white/60'}`}>TRÁFICO</button>
                            </div>
                            <button onClick={() => setShowSettings(false)} className="text-white/20 hover:text-red-400 transition-colors"><Trash2 size={16} /></button>
                        </div>
                        <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
                            {activeTab === 'keys' ? (
                                <div className="space-y-8">
                                    <div>
                                        <div className="flex justify-between items-end mb-2">
                                            <label className="text-sentinel-blue text-[10px] font-bold uppercase flex items-center gap-2"><Activity size={12} /> Pool de Google Gemini</label>
                                            <span className="text-[8px] text-white/20 uppercase tracking-tighter">separado por comas para balanceo multi-cuenta</span>
                                        </div>
                                        <textarea spellCheck={false} className="w-full bg-black/40 border border-white/10 rounded-lg p-4 text-xs text-sentinel-blue h-32 outline-none focus:border-sentinel-blue/40 transition-all font-mono" value={configKeys.gemini_api_keys || ''} onChange={e => setConfigKeys({ ...configKeys, gemini_api_keys: e.target.value })} placeholder="AIzaSy..., AIzaSy..." />
                                    </div>
                                    <div className="grid grid-cols-2 gap-6">
                                        <div>
                                            <label className="text-white/40 text-[10px] block mb-2 uppercase flex items-center gap-2"><Key size={12} /> Perplexity</label>
                                            <input spellCheck={false} className="w-full bg-black/40 border border-white/10 rounded-lg p-3 text-xs text-white outline-none focus:border-white/20 transition-all" value={configKeys.perplexity_api_key || ''} onChange={e => setConfigKeys({ ...configKeys, perplexity_api_key: e.target.value })} placeholder="pplx-..." />
                                        </div>
                                        <div>
                                            <label className="text-white/40 text-[10px] block mb-2 uppercase flex items-center gap-2"><Key size={12} /> Groq</label>
                                            <input spellCheck={false} className="w-full bg-black/40 border border-white/10 rounded-lg p-3 text-xs text-white outline-none focus:border-white/20 transition-all" value={configKeys.groq_api_key || ''} onChange={e => setConfigKeys({ ...configKeys, groq_api_key: e.target.value })} placeholder="gsk_..." />
                                        </div>
                                        <div className="col-span-2">
                                            <label className="text-white/40 text-[10px] block mb-2 uppercase flex items-center gap-2"><Key size={12} /> OpenAI (Legacy/Codex)</label>
                                            <input spellCheck={false} className="w-full bg-black/40 border border-white/10 rounded-lg p-3 text-xs text-white outline-none focus:border-white/20 transition-all" value={configKeys.openai_api_key || ''} onChange={e => setConfigKeys({ ...configKeys, openai_api_key: e.target.value })} placeholder="sk-..." />
                                        </div>
                                    </div>
                                    <div className="p-6 rounded-xl bg-sentinel-blue/5 border border-sentinel-blue/10 space-y-4">
                                        <h3 className="text-sentinel-blue font-bold uppercase text-[10px] tracking-widest flex items-center gap-2"><Shield size={14} /> Vertex AI (Empresarial)</h3>
                                        <div className="grid grid-cols-2 gap-4">
                                            <input spellCheck={false} className="bg-black/40 border border-white/10 rounded-lg p-3 text-xs text-white" placeholder="Project ID" value={configKeys.gcloud_project_id || ''} onChange={e => setConfigKeys({ ...configKeys, gcloud_project_id: e.target.value })} />
                                            <input spellCheck={false} className="bg-black/40 border border-white/10 rounded-lg p-3 text-xs text-white" placeholder="Región (us-central1)" value={configKeys.gcloud_region || ''} onChange={e => setConfigKeys({ ...configKeys, gcloud_region: e.target.value })} />
                                        </div>
                                    </div>
                                </div>
                            ) : (
                                <div className="space-y-4">
                                    {usageStats ? (
                                        Object.entries(usageStats.provider_stats).map(([k, s]) => (
                                            <div key={k} className="flex justify-between items-center p-4 bg-white/5 rounded-xl border border-white/5">
                                                <span className="text-sentinel-blue font-black uppercase text-xs">{k}</span>
                                                <div className="flex gap-8 text-[10px] font-mono">
                                                    <span className="text-white/40">REQ: <span className="text-white">{s.requests}</span></span>
                                                    <span className="text-red-400">ERR: <span>{s.errors}</span></span>
                                                    <span className={`${s.last_status === 'OK' ? 'text-sentinel-green' : 'text-red-400'}`}>{s.last_status}</span>
                                                </div>
                                            </div>
                                        ))
                                    ) : (
                                        <div className="text-white/20 text-center py-20 uppercase font-black tracking-widest text-xs">Esperando telemetría...</div>
                                    )}
                                </div>
                            )}
                        </div>
                        <div className="p-6 bg-black/30 border-t border-white/5 flex justify-end gap-3">
                            <button onClick={() => setShowSettings(false)} className="px-6 py-2 rounded-lg text-[10px] font-black uppercase text-white/40 hover:text-white transition-all">Cancelar</button>
                            <button onClick={saveSettings} className="bg-sentinel-blue text-black px-8 py-2 rounded-lg text-[10px] font-black uppercase tracking-widest hover:scale-105 transition-all shadow-[0_0_20px_rgba(0,255,249,0.2)]">Actualizar Configuración</button>
                        </div>
                    </motion.div>
                </div>
            )}
        </div>
    );
};

export default Chat;
