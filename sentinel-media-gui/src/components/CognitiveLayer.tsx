import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion, AnimatePresence } from 'framer-motion';
import {
    Brain,
    Save,
    RefreshCw,
    FileText,
    AlertCircle,
    CheckCircle,
    UserCircle,
    Cpu,
    Zap,
    History
} from 'lucide-react';

interface SystemPromptInfo {
    name: string;
    filename: string;
    path: string;
}

const CognitiveLayer: React.FC = () => {
    const [prompts, setPrompts] = useState<SystemPromptInfo[]>([]);
    const [selectedPrompt, setSelectedPrompt] = useState<SystemPromptInfo | null>(null);
    const [content, setContent] = useState<string>('');
    const [loading, setLoading] = useState(false);
    const [saving, setSaving] = useState(false);
    const [successMsg, setSuccessMsg] = useState<string | null>(null);
    const [errorMsg, setErrorMsg] = useState<string | null>(null);

    const fetchPrompts = async () => {
        setLoading(true);
        try {
            const list = await invoke<SystemPromptInfo[]>('get_prompts_sistema');
            setPrompts(list);
            if (list.length > 0 && !selectedPrompt) {
                handleSelectPrompt(list[0]);
            }
        } catch (e) {
            console.error(e);
            setErrorMsg('Error al cargar la lista de personalidades');
        } finally {
            setLoading(false);
        }
    };

    const handleSelectPrompt = async (prompt: SystemPromptInfo) => {
        setSelectedPrompt(prompt);
        setLoading(true);
        try {
            const data = await invoke<string>('leer_prompt_sistema', { filename: prompt.filename });
            setContent(data);
        } catch (e) {
            console.error(e);
            setErrorMsg('Error al leer el archivo de personalidad');
        } finally {
            setLoading(false);
        }
    };

    const handleSave = async () => {
        if (!selectedPrompt) return;
        setSaving(true);
        try {
            await invoke('guardar_prompt_sistema', { filename: selectedPrompt.filename, content });
            setSuccessMsg('Personalidad guardada en la matriz');
            setTimeout(() => setSuccessMsg(null), 3000);
        } catch (e) {
            console.error(e);
            setErrorMsg('Fallo en la sincronización de la capa cognitiva');
        } finally {
            setSaving(false);
        }
    };

    useEffect(() => {
        fetchPrompts();
    }, []);

    return (
        <div className="p-8 h-full overflow-hidden flex flex-col bg-cyber-dark text-white selection:bg-sentinel-blue/30 antialiased">
            <header className="flex justify-between items-center mb-8 shrink-0">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <Brain className="text-sentinel-blue" size={24} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase">CAPAS <span className="text-white/20">COGNITIVAS</span></h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase font-mono">Personalidad de Agentes // Orquestación de Identidad</p>
                </div>

                <button
                    onClick={fetchPrompts}
                    className="p-3 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all text-white/40 hover:text-sentinel-blue"
                >
                    <RefreshCw size={18} className={loading && !content ? "animate-spin" : ""} />
                </button>
            </header>

            <div className="flex-1 grid grid-cols-12 gap-8 min-h-0">
                {/* Selector de Personalidades */}
                <div className="col-span-3 flex flex-col gap-3 min-h-0 overflow-y-auto pr-2 custom-scrollbar">
                    <p className="text-[9px] font-black text-white/10 uppercase tracking-[0.4em] mb-2 px-2">Identidades Disponibles</p>
                    {prompts.map((p) => (
                        <button
                            key={p.filename}
                            onClick={() => handleSelectPrompt(p)}
                            className={`group w-full text-left p-4 rounded-2xl flex flex-col gap-2 transition-all relative overflow-hidden ${selectedPrompt?.filename === p.filename
                                ? 'bg-sentinel-blue/10 border border-sentinel-blue/30 text-white shadow-[0_0_20px_rgba(0,217,255,0.1)]'
                                : 'hover:bg-white/5 border border-white/5 text-white/40'
                                }`}
                        >
                            <div className="flex items-center justify-between">
                                <span className="text-[10px] font-black uppercase tracking-widest">{p.name}</span>
                                {selectedPrompt?.filename === p.filename && <Zap size={10} className="text-sentinel-blue" />}
                            </div>
                            <div className="flex items-center gap-2 opacity-30 group-hover:opacity-60 transition-opacity">
                                <FileText size={10} />
                                <span className="text-[8px] font-mono truncate">{p.filename}</span>
                            </div>
                        </button>
                    ))}
                </div>

                {/* Editor de Personalidad */}
                <div className="col-span-9 flex flex-col gap-6 min-h-0">
                    <div className="flex-1 bg-white/[0.02] border border-white/5 rounded-[2.5rem] p-8 flex flex-col relative overflow-hidden">
                        <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-[0.02] pointer-events-none" />

                        <header className="flex justify-between items-center mb-6 relative z-10">
                            <div className="flex items-center gap-4">
                                <div className="p-3 rounded-xl bg-sentinel-blue/10 border border-sentinel-blue/20 text-sentinel-blue">
                                    <UserCircle size={20} />
                                </div>
                                <div>
                                    <h3 className="text-sm font-black uppercase tracking-widest text-sentinel-blue">
                                        {selectedPrompt ? selectedPrompt.name : (loading ? 'Identificando Capa...' : 'Selecciona una Identidad')}
                                    </h3>
                                    <p className="text-[9px] font-bold text-white/20 uppercase tracking-wider font-mono">
                                        System Instruction // Markdown v4.0
                                    </p>
                                </div>
                            </div>

                            <div className="flex items-center gap-3">
                                <div className="flex items-center gap-1.5 px-3 py-1.5 bg-white/5 rounded-lg border border-white/5 text-[8px] font-black text-white/30 uppercase tracking-widest">
                                    <History size={10} /> v8.6 Sync
                                </div>
                                <button
                                    onClick={handleSave}
                                    disabled={saving || !selectedPrompt}
                                    className="flex items-center gap-2 px-6 py-2.5 rounded-xl bg-sentinel-blue text-cyber-dark text-[10px] font-black tracking-widest uppercase hover:bg-white transition-all disabled:opacity-30"
                                >
                                    {saving ? <RefreshCw size={14} className="animate-spin" /> : <Save size={14} />}
                                    SINCRONIZAR
                                </button>
                            </div>
                        </header>

                        <div className="flex-1 relative z-10 min-h-0">
                            <textarea
                                value={content}
                                onChange={(e) => setContent(e.target.value)}
                                placeholder="Cargando configuración de personalidad..."
                                className="w-full h-full bg-white/[0.03] border border-white/5 rounded-2xl p-6 text-xs text-white/60 font-mono leading-relaxed focus:outline-none focus:border-sentinel-blue/30 transition-colors resize-none custom-scrollbar"
                                spellCheck={false}
                            />
                        </div>

                        <footer className="mt-4 flex justify-between items-center relative z-10 px-2">
                            <div className="flex items-center gap-4">
                                <div className="flex items-center gap-2">
                                    <div className="w-1.5 h-1.5 rounded-full bg-sentinel-green animate-pulse" />
                                    <span className="text-[8px] font-black text-white/20 uppercase tracking-widest">Capa Activa</span>
                                </div>
                                <div className="flex items-center gap-2">
                                    <Cpu size={10} className="text-white/10" />
                                    <span className="text-[8px] font-black text-white/20 uppercase tracking-widest">Local Matrix</span>
                                </div>
                            </div>

                            <AnimatePresence>
                                {successMsg && (
                                    <motion.div
                                        initial={{ opacity: 0, x: 20 }}
                                        animate={{ opacity: 1, x: 0 }}
                                        exit={{ opacity: 0 }}
                                        className="flex items-center gap-2 text-sentinel-green text-[9px] font-black uppercase tracking-widest"
                                    >
                                        <CheckCircle size={12} />
                                        {successMsg}
                                    </motion.div>
                                )}
                                {errorMsg && (
                                    <motion.div
                                        initial={{ opacity: 0, x: 20 }}
                                        animate={{ opacity: 1, x: 0 }}
                                        exit={{ opacity: 0 }}
                                        className="flex items-center gap-2 text-red-400 text-[9px] font-black uppercase tracking-widest"
                                    >
                                        <AlertCircle size={12} />
                                        {errorMsg}
                                    </motion.div>
                                )}
                            </AnimatePresence>
                        </footer>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default CognitiveLayer;
