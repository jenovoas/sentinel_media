import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    Terminal,
    Search,
    Upload,
    Zap,
    RefreshCw,
    FileText,
    Video,
    Image as ImageIcon,
    CheckCircle,
    AlertCircle,
    Loader
} from 'lucide-react';
import { motion } from 'framer-motion';

interface CommandResult {
    command: string;
    output: string;
    status: 'success' | 'error' | 'running';
    timestamp: string;
}

const CommandPalette: React.FC = () => {
    const [results, setResults] = useState<CommandResult[]>([]);
    const [running, setRunning] = useState<string | null>(null);

    const executeCommand = async (cmd: string, label: string) => {
        setRunning(cmd);

        const newResult: CommandResult = {
            command: label,
            output: '',
            status: 'running',
            timestamp: new Date().toLocaleTimeString()
        };

        setResults(prev => [newResult, ...prev]);

        try {
            const output = await invoke<string>('execute_sentinel_command', { command: cmd });
            setResults(prev => prev.map((r, i) =>
                i === 0 ? { ...r, output, status: 'success' } : r
            ));
        } catch (e) {
            setResults(prev => prev.map((r, i) =>
                i === 0 ? { ...r, output: String(e), status: 'error' } : r
            ));
        } finally {
            setRunning(null);
        }
    };

    const commands = [
        {
            id: 'scan',
            label: 'Escanear Bóveda',
            description: 'Buscar notas listas para producción',
            icon: Search,
            command: 'factory scan',
            color: 'sentinel-blue'
        },
        {
            id: 'research',
            label: 'Investigar',
            description: 'Iniciar investigación profunda',
            icon: FileText,
            command: 'research --deep',
            color: 'sentinel-green'
        },
        {
            id: 'generate-short',
            label: 'Generar Short',
            description: 'Crear video corto (9:16)',
            icon: Video,
            command: 'factory generate --type short',
            color: 'purple-500'
        },
        {
            id: 'generate-long',
            label: 'Generar Longform',
            description: 'Crear video largo (16:9)',
            icon: Video,
            command: 'factory generate --type longform',
            color: 'purple-500'
        },
        {
            id: 'generate-image',
            label: 'Generar Imagen',
            description: 'Crear diagrama técnico',
            icon: ImageIcon,
            command: 'media image',
            color: 'pink-500'
        },
        {
            id: 'index',
            label: 'Indexar Biblioteca',
            description: 'Actualizar índice de la bóveda',
            icon: RefreshCw,
            command: 'library index',
            color: 'yellow-500'
        },
        {
            id: 'publish',
            label: 'Publicar',
            description: 'Subir contenido a YouTube',
            icon: Upload,
            command: 'publish',
            color: 'red-500'
        },
        {
            id: 'status',
            label: 'Estado del Sistema',
            description: 'Ver estado de operaciones',
            icon: Zap,
            command: 'status',
            color: 'sentinel-blue'
        }
    ];

    return (
        <div className="flex flex-col h-full overflow-hidden p-8">
            <header className="bg-white/[0.02] p-8 rounded-[2.5rem] border border-white/5 mb-8">
                <div className="flex items-center gap-3 mb-2">
                    <Terminal size={24} className="text-sentinel-blue" />
                    <h1 className="text-3xl font-black text-white uppercase tracking-tighter">
                        PANEL DE <span className="text-white/40">COMANDOS</span>
                    </h1>
                </div>
                <p className="text-white/30 text-[10px] font-bold uppercase tracking-[0.4em] font-mono">
                    Ejecución Directa de Operaciones Sentinel
                </p>
            </header>

            <div className="grid grid-cols-2 gap-6 mb-8">
                {commands.map((cmd) => {
                    const Icon = cmd.icon;
                    const isRunning = running === cmd.command;

                    return (
                        <motion.button
                            key={cmd.id}
                            onClick={() => executeCommand(cmd.command, cmd.label)}
                            disabled={isRunning || running !== null}
                            whileHover={{ scale: 1.02 }}
                            whileTap={{ scale: 0.98 }}
                            className={`relative group bg-white/[0.02] border border-white/10 rounded-[2rem] p-6 text-left transition-all hover:bg-white/[0.05] disabled:opacity-50 disabled:cursor-not-allowed overflow-hidden`}
                        >
                            <div className={`absolute inset-0 bg-gradient-to-br from-${cmd.color}/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity`} />

                            <div className="relative z-10">
                                <div className="flex items-start justify-between mb-4">
                                    <div className={`w-12 h-12 rounded-2xl bg-${cmd.color}/20 flex items-center justify-center`}>
                                        {isRunning ? (
                                            <Loader size={24} className={`text-${cmd.color} animate-spin`} />
                                        ) : (
                                            <Icon size={24} className={`text-${cmd.color}`} />
                                        )}
                                    </div>
                                    <div className={`text-[9px] font-black uppercase tracking-widest text-${cmd.color}/60 px-3 py-1 rounded-full bg-${cmd.color}/10`}>
                                        {isRunning ? 'Ejecutando...' : 'Listo'}
                                    </div>
                                </div>

                                <h3 className="text-lg font-black text-white mb-2 uppercase">
                                    {cmd.label}
                                </h3>
                                <p className="text-sm text-white/40 font-medium">
                                    {cmd.description}
                                </p>

                                <div className="mt-4 flex items-center gap-2 text-[9px] font-black uppercase tracking-widest text-white/20">
                                    <Terminal size={10} />
                                    <code className="font-mono">sentinel {cmd.command}</code>
                                </div>
                            </div>
                        </motion.button>
                    );
                })}
            </div>

            {/* Resultados */}
            <div className="flex-1 overflow-hidden">
                <h2 className="text-sm font-black uppercase tracking-widest text-white/40 mb-4">
                    Historial de Ejecución
                </h2>

                <div className="h-full overflow-y-auto space-y-4 pr-4">
                    {results.length === 0 ? (
                        <div className="h-full flex items-center justify-center text-white/20">
                            <div className="text-center">
                                <Terminal size={48} className="mx-auto mb-4 opacity-20" />
                                <p className="text-sm font-medium">No hay comandos ejecutados aún</p>
                            </div>
                        </div>
                    ) : (
                        results.map((result, idx) => (
                            <motion.div
                                key={idx}
                                initial={{ opacity: 0, y: -10 }}
                                animate={{ opacity: 1, y: 0 }}
                                className="bg-white/[0.02] border border-white/5 rounded-2xl p-6"
                            >
                                <div className="flex items-start justify-between mb-4">
                                    <div className="flex items-center gap-3">
                                        {result.status === 'success' && (
                                            <CheckCircle size={20} className="text-sentinel-green" />
                                        )}
                                        {result.status === 'error' && (
                                            <AlertCircle size={20} className="text-red-500" />
                                        )}
                                        {result.status === 'running' && (
                                            <Loader size={20} className="text-sentinel-blue animate-spin" />
                                        )}
                                        <span className="font-black text-white uppercase text-sm">
                                            {result.command}
                                        </span>
                                    </div>
                                    <span className="text-[9px] font-black uppercase tracking-widest text-white/20">
                                        {result.timestamp}
                                    </span>
                                </div>

                                {result.output && (
                                    <pre className="bg-black/20 rounded-xl p-4 text-xs font-mono text-white/60 overflow-x-auto whitespace-pre-wrap">
                                        {result.output}
                                    </pre>
                                )}
                            </motion.div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
};

export default CommandPalette;
