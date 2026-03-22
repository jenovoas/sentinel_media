import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { Settings, Shield, Terminal, Save, Key, DollarSign, CheckCircle } from 'lucide-react';

interface CostSummary {
    total_today: number;
    total_this_month: number;
    daily_budget: number;
    monthly_budget: number;
    daily_budget_usage_pct: number;
    monthly_budget_usage_pct: number;
    alert_threshold_pct: number;
}

const SettingsView: React.FC = () => {
    const [activeTab, setActiveTab] = useState<'billing' | 'security' | 'system' | 'prompts'>('billing');
    const [loading, setLoading] = useState(false);
    const [successMsg, setSuccessMsg] = useState<string | null>(null);

    // Estado de facturacion
    const [dailyBudget, setDailyBudget] = useState(50);
    const [monthlyBudget, setMonthlyBudget] = useState(1000);
    const [alertThreshold, setAlertThreshold] = useState(80);

    // Estado de llaves API y uso
    const [configKeys, setConfigKeys] = useState<any>({});

    // Estado de prompts de sistema
    const [systemPrompts, setSystemPrompts] = useState<string[]>([]);
    const [selectedPrompt, setSelectedPrompt] = useState<string | null>(null);
    const [promptContent, setPromptContent] = useState<string>('');

    useEffect(() => {
        fetchSettings();
    }, []);

    const fetchSettings = async () => {
        try {
            const summary = await invoke<CostSummary>('get_resumen_costos');
            setDailyBudget(summary.daily_budget || 50);
            setMonthlyBudget(summary.monthly_budget || 1000);
            setAlertThreshold(summary.alert_threshold_pct || 80);

            const keys = await invoke<any>('get_llaves_api');
            setConfigKeys(keys);

            // const usage = await invoke<any>('obtener_uso_sentinomica');
            // setUsageStats(usage);

            // Obtener prompts de sistema
            try {
                const prompts = await invoke<string[]>('get_prompts_sistema');
                setSystemPrompts(prompts);
            } catch (e) {
                console.error('Error obteniendo prompts:', e);
            }
        } catch (e) {
            console.error('Error obteniendo configuracion:', e);
        }
    };

    const handleSaveBudget = async () => {
        setLoading(true);
        try {
            await invoke('establecer_presupuesto', {
                daily: dailyBudget,
                monthly: monthlyBudget,
                threshold: alertThreshold
            });
            setSuccessMsg('Presupuesto actualizado correctamente');
            setTimeout(() => setSuccessMsg(null), 3000);
            await fetchSettings();
        } catch (e) {
            console.error('Error guardando presupuesto:', e);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="p-8 h-full overflow-hidden flex flex-col bg-cyber-dark text-white selection:bg-sentinel-blue/30 antialiased">
            <header className="flex justify-between items-center mb-8 shrink-0">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <Settings className="text-sentinel-blue" size={24} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase">CONFIGURACIÓN <span className="text-white/20">DEL CORTEX</span></h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase font-mono">System Parameters // Security // Cost Control</p>
                </div>
            </header>

            <div className="flex-1 grid grid-cols-12 gap-8 min-h-0">
                {/* Sidebar de Configuración */}
                <div className="col-span-3 space-y-2">
                    {[
                        { id: 'billing', label: 'Costos & Presupuesto', icon: DollarSign },
                        { id: 'security', label: 'API Keys & Secretos', icon: Key },
                        { id: 'prompts', label: 'System Prompts', icon: Terminal },
                        { id: 'system', label: 'System Diagnostics', icon: Shield },
                    ].map((tab) => (
                        <button
                            key={tab.id}
                            onClick={() => setActiveTab(tab.id as any)}
                            className={`w-full text-left px-4 py-4 rounded-xl flex items-center gap-3 transition-all ${activeTab === tab.id
                                ? 'bg-sentinel-blue/10 border border-sentinel-blue/30 text-white shadow-[0_0_15px_rgba(0,240,255,0.1)]'
                                : 'hover:bg-white/5 border border-transparent text-white/40 hover:text-white'
                                }`}
                        >
                            <tab.icon size={18} className={activeTab === tab.id ? 'text-sentinel-blue' : 'text-current'} />
                            <span className="text-xs font-bold uppercase tracking-wider">{tab.label}</span>
                        </button>
                    ))}
                </div>

                {/* Panel Principal */}
                <div className="col-span-9 bg-white/[0.02] border border-white/5 rounded-[2.5rem] p-8 overflow-y-auto custom-scrollbar relative">
                    <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-[0.02] pointer-events-none" />

                    {activeTab === 'billing' && (
                        <motion.div
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="space-y-8 relative z-10"
                        >
                            <div className="flex justify-between items-center border-b border-white/10 pb-6">
                                <div>
                                    <h2 className="text-xl font-black uppercase tracking-wide">Control de Presupuesto</h2>
                                    <p className="text-xs text-white/40 mt-1">Define los límites duros para la generación de contenido AI.</p>
                                </div>
                                <div className="px-3 py-1 bg-sentinel-blue/10 rounded-lg border border-sentinel-blue/20 text-sentinel-blue text-[10px] font-mono font-bold">
                                    AUTO-BALANCE ENABLED
                                </div>
                            </div>

                            <div className="grid grid-cols-2 gap-8">
                                <div className="space-y-4">
                                    <label className="text-xs font-bold text-white/60 uppercase tracking-wider flex justify-between">
                                        Límite Diario (USD)
                                        <span className="text-sentinel-blue font-mono">${dailyBudget}</span>
                                    </label>
                                    <input
                                        type="range"
                                        min="1" max="200" step="1"
                                        value={dailyBudget}
                                        onChange={(e) => setDailyBudget(Number(e.target.value))}
                                        className="w-full h-2 bg-white/10 rounded-lg appearance-none cursor-pointer accent-sentinel-blue hover:accent-sentinel-blue/80"
                                    />
                                    <div className="flex justify-between text-[10px] text-white/20 font-mono">
                                        <span>$1</span>
                                        <span>$200</span>
                                    </div>
                                </div>

                                <div className="space-y-4">
                                    <label className="text-xs font-bold text-white/60 uppercase tracking-wider flex justify-between">
                                        Límite Mensual (USD)
                                        <span className="text-sentinel-blue font-mono">${monthlyBudget}</span>
                                    </label>
                                    <input
                                        type="range"
                                        min="50" max="5000" step="50"
                                        value={monthlyBudget}
                                        onChange={(e) => setMonthlyBudget(Number(e.target.value))}
                                        className="w-full h-2 bg-white/10 rounded-lg appearance-none cursor-pointer accent-sentinel-blue hover:accent-sentinel-blue/80"
                                    />
                                    <div className="flex justify-between text-[10px] text-white/20 font-mono">
                                        <span>$50</span>
                                        <span>$5000</span>
                                    </div>
                                </div>
                            </div>

                            <div className="p-6 rounded-2xl bg-white/[0.03] border border-white/10 space-y-4">
                                <label className="text-xs font-bold text-white/60 uppercase tracking-wider flex justify-between">
                                    Umbral de Alerta (%)
                                    <span className="text-yellow-500 font-mono">{alertThreshold}%</span>
                                </label>
                                <p className="text-[10px] text-white/30 mb-4">El sistema pausará operaciones no críticas cuando se alcance este porcentaje del presupuesto diario.</p>
                                <input
                                    type="range"
                                    min="50" max="100" step="5"
                                    value={alertThreshold}
                                    onChange={(e) => setAlertThreshold(Number(e.target.value))}
                                    className="w-full h-2 bg-white/10 rounded-lg appearance-none cursor-pointer accent-yellow-500 hover:accent-yellow-400"
                                />
                            </div>

                            <div className="flex justify-end pt-4">
                                <button
                                    onClick={handleSaveBudget}
                                    disabled={loading}
                                    className="flex items-center gap-2 px-8 py-3 rounded-xl bg-sentinel-blue text-cyber-dark text-xs font-black tracking-widest uppercase hover:bg-white transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {loading ? (
                                        <div className="w-4 h-4 border-2 border-cyber-dark border-t-transparent rounded-full animate-spin" />
                                    ) : (
                                        <Save size={16} />
                                    )}
                                    Grid Save
                                </button>
                            </div>

                            {successMsg && (
                                <motion.div
                                    initial={{ opacity: 0, y: 10 }}
                                    animate={{ opacity: 1, y: 0 }}
                                    className="flex items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-xl text-green-500 text-xs font-bold justify-center"
                                >
                                    <CheckCircle size={14} />
                                    {successMsg}
                                </motion.div>
                            )}
                        </motion.div>
                    )}

                    {activeTab === 'security' && (
                        <motion.div
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="space-y-6 relative z-10"
                        >
                            <div className="flex justify-between items-center border-b border-white/10 pb-6">
                                <div>
                                    <h2 className="text-xl font-black uppercase tracking-wide">Bóveda de Credenciales</h2>
                                    <p className="text-xs text-white/40 mt-1">Configuración maestra de llaves API y Secretos del Sistema.</p>
                                </div>
                                <Shield className="text-sentinel-blue" size={20} />
                            </div>

                            <div className="space-y-6">
                                {/* Google Gemini Pools (Token Balancer) */}
                                <div className="space-y-4">
                                    <div className="flex justify-between items-end">
                                        <label className="text-[10px] font-bold text-white/60 uppercase tracking-widest flex items-center gap-2">
                                            <div className="w-2 h-2 rounded-full bg-sentinel-blue shadow-[0_0_8px_rgba(0,217,255,0.6)]" />
                                            Token Load Balancer (Google Gemini)
                                        </label>
                                        <div className="text-[9px] font-mono text-white/30">
                                            Active Nodes: <span className="text-sentinel-blue font-bold">
                                                {configKeys.gemini_api_keys ? configKeys.gemini_api_keys.split(',').filter((k: string) => k.trim().length > 0).length : 0}
                                            </span>
                                        </div>
                                    </div>

                                    <div className="p-4 rounded-2xl bg-black/20 border border-white/5 space-y-3">
                                        <div className="grid grid-cols-1 gap-2 max-h-48 overflow-y-auto custom-scrollbar pr-2">
                                            {configKeys.gemini_api_keys?.split(',').filter((k: string) => k.trim().length > 0).map((key: string, idx: number) => (
                                                <div key={idx} className="flex items-center justify-between p-3 rounded-xl bg-white/5 border border-white/5 group hover:border-sentinel-blue/30 transition-all">
                                                    <div className="flex items-center gap-3">
                                                        <div className="w-6 h-6 rounded-lg bg-sentinel-blue/10 flex items-center justify-center text-sentinel-blue">
                                                            <Key size={12} />
                                                        </div>
                                                        <div className="flex flex-col">
                                                            <span className="text-[10px] font-mono text-white/70">
                                                                {key.trim().substring(0, 8)}...{key.trim().substring(key.trim().length - 4)}
                                                            </span>
                                                            <span className="text-[8px] font-bold uppercase text-sentinel-green">Active Node #{idx + 1}</span>
                                                        </div>
                                                    </div>
                                                    <button
                                                        onClick={() => {
                                                            const newKeys = configKeys.gemini_api_keys.split(',').filter((_: string, i: number) => i !== idx).join(',');
                                                            setConfigKeys({ ...configKeys, gemini_api_keys: newKeys });
                                                        }}
                                                        className="p-1.5 rounded-lg text-white/20 hover:text-red-400 hover:bg-red-400/10 transition-colors"
                                                    >
                                                        <div className="w-3 h-3 relative">
                                                            <div className="absolute top-1.5 left-0 w-3 h-px bg-current rotate-45" />
                                                            <div className="absolute top-1.5 left-0 w-3 h-px bg-current -rotate-45" />
                                                        </div>
                                                    </button>
                                                </div>
                                            ))}
                                            {(configKeys.gemini_api_keys?.split(',').filter((k: string) => k.trim().length > 0).length || 0) === 0 && (
                                                <div className="text-center py-4 text-[10px] text-white/20 italic">
                                                    Pool vacío. Agrega llaves para activar el balanceo de carga.
                                                </div>
                                            )}
                                        </div>

                                        <div className="flex items-center gap-2 pt-2 border-t border-white/5">
                                            <input
                                                type="text"
                                                id="new-gemini-key"
                                                placeholder="Pegar nueva AIzaSy... key"
                                                className="flex-1 bg-transparent border-none text-xs text-white placeholder-white/10 focus:ring-0 outline-none font-mono"
                                                onKeyDown={(e) => {
                                                    if (e.key === 'Enter') {
                                                        const input = e.currentTarget;
                                                        if (input.value.trim().length > 0) {
                                                            const current = configKeys.gemini_api_keys ? configKeys.gemini_api_keys : "";
                                                            const separator = current.length > 0 ? "," : "";
                                                            setConfigKeys({ ...configKeys, gemini_api_keys: current + separator + input.value.trim() });
                                                            input.value = "";
                                                        }
                                                    }
                                                }}
                                            />
                                            <button
                                                onClick={() => {
                                                    const input = document.getElementById('new-gemini-key') as HTMLInputElement;
                                                    if (input && input.value.trim().length > 0) {
                                                        const current = configKeys.gemini_api_keys ? configKeys.gemini_api_keys : "";
                                                        const separator = current.length > 0 ? "," : "";
                                                        setConfigKeys({ ...configKeys, gemini_api_keys: current + separator + input.value.trim() });
                                                        input.value = "";
                                                    }
                                                }}
                                                className="px-3 py-1.5 rounded-lg bg-sentinel-blue/10 border border-sentinel-blue/20 text-sentinel-blue text-[9px] font-black uppercase hover:bg-sentinel-blue hover:text-cyber-dark transition-all"
                                            >
                                                Add Node
                                            </button>
                                        </div>
                                    </div>
                                    <p className="text-[9px] text-white/20 ml-2">El sistema rotará automáticamente entre estos nodos (Round-Robin) para maximizar quota.</p>
                                </div>

                                <div className="grid grid-cols-2 gap-6">
                                    <div className="space-y-2">
                                        <label className="text-[10px] font-bold text-white/60 uppercase tracking-widest block">Perplexity API Key</label>
                                        <div className="relative">
                                            <input
                                                type="password"
                                                value={configKeys.perplexity_api_key || ''}
                                                onChange={(e) => setConfigKeys({ ...configKeys, perplexity_api_key: e.target.value })}
                                                placeholder="pplx-..."
                                                className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-xs font-mono text-white placeholder-white/10 focus:border-white/30 outline-none transition-all"
                                            />
                                            <Key size={12} className="absolute right-4 top-3.5 text-white/10" />
                                        </div>
                                    </div>

                                    <div className="space-y-2">
                                        <label className="text-[10px] font-bold text-white/60 uppercase tracking-widest block">Groq Cloud API Key</label>
                                        <div className="relative">
                                            <input
                                                type="password"
                                                value={configKeys.groq_api_key || ''}
                                                onChange={(e) => setConfigKeys({ ...configKeys, groq_api_key: e.target.value })}
                                                placeholder="gsk_..."
                                                className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-xs font-mono text-white placeholder-white/10 focus:border-white/30 outline-none transition-all"
                                            />
                                            <Key size={12} className="absolute right-4 top-3.5 text-white/10" />
                                        </div>
                                    </div>

                                    <div className="col-span-2 space-y-2">
                                        <label className="text-[10px] font-bold text-white/60 uppercase tracking-widest block">OpenAI / Compatible Key</label>
                                        <div className="relative">
                                            <input
                                                type="password"
                                                value={configKeys.openai_api_key || ''}
                                                onChange={(e) => setConfigKeys({ ...configKeys, openai_api_key: e.target.value })}
                                                placeholder="sk-..."
                                                className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-xs font-mono text-white placeholder-white/10 focus:border-white/30 outline-none transition-all"
                                            />
                                            <Key size={12} className="absolute right-4 top-3.5 text-white/10" />
                                        </div>
                                    </div>
                                </div>

                                {/* Vertex AI Section */}
                                <div className="p-6 rounded-2xl bg-white/[0.02] border border-white/5 space-y-4">
                                    <div className="flex items-center gap-3 mb-2">
                                        <div className="p-2 bg-yellow-500/10 rounded-lg text-yellow-500">
                                            <Shield size={16} />
                                        </div>
                                        <div>
                                            <h3 className="text-xs font-black uppercase tracking-widest text-white">Google Cloud Vertex AI</h3>
                                            <p className="text-[9px] text-white/30">Configuración empresarial para modelos Gemini Pro/Vision</p>
                                        </div>
                                    </div>
                                    <div className="grid grid-cols-2 gap-6">
                                        <div className="space-y-2">
                                            <label className="text-[10px] font-bold text-white/40 uppercase tracking-widest block">Project ID</label>
                                            <input
                                                type="text"
                                                value={configKeys.gcloud_project_id || ''}
                                                onChange={(e) => setConfigKeys({ ...configKeys, gcloud_project_id: e.target.value })}
                                                placeholder="my-gcp-project-id"
                                                className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-xs font-mono text-white placeholder-white/10 focus:border-yellow-500/30 outline-none transition-all"
                                            />
                                        </div>
                                        <div className="space-y-2">
                                            <label className="text-[10px] font-bold text-white/40 uppercase tracking-widest block">Region Location</label>
                                            <input
                                                type="text"
                                                value={configKeys.gcloud_region || ''}
                                                onChange={(e) => setConfigKeys({ ...configKeys, gcloud_region: e.target.value })}
                                                placeholder="us-central1"
                                                className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-xs font-mono text-white placeholder-white/10 focus:border-yellow-500/30 outline-none transition-all"
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div className="flex justify-end pt-6 border-t border-white/5">
                                <button
                                    onClick={async () => {
                                        setLoading(true);
                                        try {
                                            await invoke('guardar_llaves_api', { keys: configKeys });
                                            setSuccessMsg('Llaves maestras actualizadas y cifradas.');
                                            setTimeout(() => setSuccessMsg(null), 3000);
                                        } catch (e) {
                                            console.error(e);
                                            alert("Error guardando llaves: " + e);
                                        } finally {
                                            setLoading(false);
                                        }
                                    }}
                                    disabled={loading}
                                    className="flex items-center gap-2 px-8 py-3 rounded-xl bg-sentinel-blue text-cyber-dark text-xs font-black tracking-widest uppercase hover:bg-white transition-all disabled:opacity-50 hover:shadow-[0_0_20px_rgba(0,217,255,0.4)]"
                                >
                                    {loading ? <div className="w-4 h-4 border-2 border-cyber-dark border-t-transparent rounded-full animate-spin" /> : <Save size={16} />}
                                    Actualizar Bóveda
                                </button>
                            </div>

                            {successMsg && (
                                <motion.div
                                    initial={{ opacity: 0, y: 10 }}
                                    animate={{ opacity: 1, y: 0 }}
                                    className="flex items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-xl text-green-500 text-xs font-bold justify-center mt-4"
                                >
                                    <CheckCircle size={14} />
                                    {successMsg}
                                </motion.div>
                            )}
                        </motion.div>
                    )}

                    {activeTab === 'prompts' && (
                        <motion.div
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="space-y-6 relative z-10"
                        >
                            <div className="flex justify-between items-center border-b border-white/10 pb-6">
                                <div>
                                    <h2 className="text-xl font-black uppercase tracking-wide">System Prompts Editor</h2>
                                    <p className="text-xs text-white/40 mt-1">Edita los prompts del sistema para cada agente.</p>
                                </div>
                                <Terminal className="text-sentinel-blue" size={20} />
                            </div>

                            <div className="grid grid-cols-12 gap-6">
                                <div className="col-span-4 space-y-2 max-h-[500px] overflow-y-auto custom-scrollbar pr-2">
                                    {systemPrompts.map((prompt, idx) => (
                                        <button
                                            key={idx}
                                            onClick={async () => {
                                                setSelectedPrompt(prompt);
                                                try {
                                                    const content = await invoke<string>('leer_prompt_sistema', { filename: prompt });
                                                    setPromptContent(content);
                                                } catch (e) {
                                                    console.error('Error leyendo prompt:', e);
                                                }
                                            }}
                                            className={`w-full text-left p-4 rounded-xl transition-all ${selectedPrompt === prompt
                                                ? 'bg-sentinel-blue/10 border border-sentinel-blue/30 text-white'
                                                : 'bg-white/5 border border-white/5 text-white/60 hover:bg-white/10 hover:text-white'
                                                }`}
                                        >
                                            <p className="text-xs font-bold font-mono">{prompt}</p>
                                        </button>
                                    ))}
                                    {systemPrompts.length === 0 && (
                                        <div className="text-center py-8 text-white/20 text-xs">No system prompts found</div>
                                    )}
                                </div>

                                <div className="col-span-8 space-y-4">
                                    {selectedPrompt ? (
                                        <>
                                            <div className="flex justify-between items-center">
                                                <h3 className="text-sm font-bold text-white/80">Editing: {selectedPrompt}</h3>
                                                <button
                                                    onClick={async () => {
                                                        setLoading(true);
                                                        try {
                                                            await invoke('guardar_prompt_sistema', { filename: selectedPrompt, content: promptContent });
                                                            setSuccessMsg('Prompt guardado correctamente');
                                                            setTimeout(() => setSuccessMsg(null), 3000);
                                                        } catch (e) {
                                                            console.error('Error guardando prompt:', e);
                                                        } finally {
                                                            setLoading(false);
                                                        }
                                                    }}
                                                    disabled={loading}
                                                    className="flex items-center gap-2 px-6 py-2 rounded-xl bg-sentinel-green/20 hover:bg-sentinel-green/30 text-sentinel-green text-xs font-bold uppercase transition-all disabled:opacity-50"
                                                >
                                                    {loading ? <div className="w-3 h-3 border-2 border-sentinel-green border-t-transparent rounded-full animate-spin" /> : <Save size={14} />}
                                                    Save Prompt
                                                </button>
                                            </div>
                                            <textarea
                                                value={promptContent}
                                                onChange={(e) => setPromptContent(e.target.value)}
                                                className="w-full h-[400px] bg-black/40 border border-white/10 rounded-xl p-4 text-xs font-mono text-white placeholder-white/20 focus:border-sentinel-blue/30 outline-none resize-none custom-scrollbar"
                                                placeholder="Prompt content..."
                                            />
                                        </>
                                    ) : (
                                        <div className="flex flex-col items-center justify-center h-[400px] text-white/20">
                                            <Terminal size={48} className="mb-4 opacity-20" />
                                            <p className="text-xs font-bold uppercase">Select a prompt to edit</p>
                                        </div>
                                    )}
                                </div>
                            </div>

                            {successMsg && (
                                <motion.div
                                    initial={{ opacity: 0, y: 10 }}
                                    animate={{ opacity: 1, y: 0 }}
                                    className="flex items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-xl text-green-500 text-xs font-bold justify-center"
                                >
                                    <CheckCircle size={14} />
                                    {successMsg}
                                </motion.div>
                            )}
                        </motion.div>
                    )}

                    {activeTab === 'system' && (
                        <div className="flex flex-col items-center justify-center h-full text-white/20">
                            <Terminal size={48} className="mb-4 opacity-20" />
                            <h3 className="text-sm font-bold uppercase tracking-widest mb-2">System Diagnostics</h3>
                            <p className="text-xs font-mono">Running Sentinel Core v8.0.1 (Rust Optimized)</p>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default SettingsView;
