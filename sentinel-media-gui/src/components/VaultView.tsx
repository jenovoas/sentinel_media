import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    Library,
    FileText,
    Search,
    Clock,
    Package,
    ArrowUpRight
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface VaultFile {
    name: string;
    path: string;
    modified_at: string;
    size_bytes: number;
}

const VaultView: React.FC = () => {
    const [files, setFiles] = useState<VaultFile[]>([]);
    const [search, setSearch] = useState('');
    const [selectedFile, setSelectedFile] = useState<VaultFile | null>(null);
    const [fileContent, setFileContent] = useState<string>('');
    const [isEditing, setIsEditing] = useState(false);
    const [toast, setToast] = useState<{ type: 'ok' | 'error'; msg: string } | null>(null);

    const showToast = (type: 'ok' | 'error', msg: string) => {
        setToast({ type, msg });
        setTimeout(() => setToast(null), 3500);
    };

    const fetchFiles = async () => {
        try {
            const res = await invoke<VaultFile[]>('get_archivos_sentinel_media');
            setFiles(res);
        } catch (e) {
            console.error('Error al obtener archivos de la sentinel_media:', e);
        }
    };

    useEffect(() => {
        fetchFiles();
    }, []);

    const filteredFiles = files.filter(f => f.name.toLowerCase().includes(search.toLowerCase()));

    return (
        <div className="p-8 space-y-8 h-full flex flex-col overflow-hidden bg-cyber-dark text-white selection:bg-sentinel-blue/30 antialiased relative">
            {toast && (
                <div className={`fixed bottom-6 right-6 z-50 px-5 py-3 rounded-2xl text-xs font-black uppercase tracking-widest shadow-lg ${toast.type === 'ok' ? 'bg-sentinel-green/20 border border-sentinel-green/40 text-sentinel-green' : 'bg-red-500/20 border border-red-500/40 text-red-400'}`}>
                    {toast.msg}
                </div>
            )}
            <header className="flex justify-between items-end bg-white/[0.02] p-8 rounded-[2.5rem] border border-white/5">
                <div>
                    <div className="flex items-center gap-3 mb-2">
                        <Library size={24} className="text-sentinel-blue" />
                        <h1 className="text-3xl font-black text-white uppercase tracking-tighter text-shadow-sm">BIBLIOTECA</h1>
                    </div>
                    <p className="text-white/30 text-[10px] font-bold uppercase tracking-[0.3em] font-mono">
                        Unidades de Conocimiento // <span className="text-sentinel-green">Sincronizado</span>
                    </p>
                </div>

                <div className="flex gap-4">
                    <button
                        onClick={async () => {
                            const nombre = prompt("Nombre del nuevo archivo:");
                            if (nombre) {
                                try {
                                    await invoke<string>('crear_nuevo_archivo_sentinel_media', { nombre });
                                    showToast('ok', `Archivo creado: ${nombre}`);
                                    fetchFiles();
                                } catch (e) {
                                    showToast('error', String(e));
                                }
                            }
                        }}
                        className="px-6 py-4 rounded-2xl bg-sentinel-blue/10 border border-sentinel-blue/20 text-sentinel-blue font-black text-[10px] uppercase tracking-widest hover:bg-sentinel-blue hover:text-cyber-dark transition-all"
                    >
                        + Nuevo Archivo
                    </button>
                    <div className="relative group">
                        <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-white/20 group-focus-within:text-sentinel-blue transition-colors" size={18} />
                        <input
                            type="text"
                            placeholder="Buscar archivos..."
                            value={search}
                            onChange={(e) => setSearch(e.target.value)}
                            className="pl-12 pr-6 py-4 rounded-2xl bg-white/5 border border-white/10 outline-none text-sm font-medium focus:border-sentinel-blue/30 focus:bg-white/[0.08] transition-all w-80"
                        />
                    </div>
                </div>
            </header>

            <div className="flex-1 overflow-hidden flex gap-8">
                <div className={`overflow-y-auto pr-2 space-y-3 custom-scrollbar transition-all duration-500 ${selectedFile ? 'flex-1' : 'w-full'}`}>
                    <div className={`grid gap-6 ${selectedFile ? 'grid-cols-1 md:grid-cols-2' : 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3'}`}>
                        <AnimatePresence>
                            {filteredFiles.map((file, idx) => (
                                <motion.div
                                    key={file.path}
                                    initial={{ opacity: 0, y: 10 }}
                                    animate={{ opacity: 1, y: 0 }}
                                    transition={{ delay: idx * 0.01 }}
                                    whileHover={{ y: -2, borderColor: 'rgba(0, 217, 255, 0.2)' }}
                                    onClick={async () => {
                                        setSelectedFile(file);
                                        try {
                                            const content = await invoke<string>('leer_contenido_archivo_sentinel_media', { path: file.path });
                                            setFileContent(content);
                                        } catch (e) {
                                            setFileContent("Error: " + e);
                                        }
                                    }}
                                    className={`p-6 rounded-[2rem] border transition-all cursor-pointer relative overflow-hidden ${selectedFile?.path === file.path ? 'bg-sentinel-blue/20 border-sentinel-blue/40 shadow-[0_0_30px_rgba(0,217,255,0.1)]' : 'bg-white/[0.02] border-white/5 hover:bg-white/10'}`}
                                >
                                    <div className="absolute top-0 right-0 p-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <ArrowUpRight size={16} className="text-sentinel-blue" />
                                    </div>

                                    <div className="flex items-start gap-4 mb-6">
                                        <div className={`p-3 rounded-2xl transition-colors ${selectedFile?.path === file.path ? 'bg-sentinel-blue/20 text-sentinel-blue' : 'bg-white/5 text-white/20 group-hover:bg-sentinel-blue/10 group-hover:text-sentinel-blue'}`}>
                                            <FileText size={20} />
                                        </div>
                                        <div>
                                            <h3 className={`text-sm font-black uppercase tracking-tight line-clamp-1 transition-colors ${selectedFile?.path === file.path ? 'text-sentinel-blue' : 'text-white/80'}`}>{file.name.replace('.md', '')}</h3>
                                            <p className="text-[9px] font-black text-white/20 uppercase tracking-widest mt-1">
                                                {(file.size_bytes / 1024).toFixed(1)} KB // Markdown
                                            </p>
                                        </div>
                                    </div>

                                    <div className="flex items-center gap-4 pt-4 border-t border-white/5">
                                        <div className="flex items-center gap-1.5 text-[9px] font-black text-white/30 uppercase tracking-widest italic">
                                            <Clock size={12} /> {file.modified_at ? new Date(file.modified_at).toLocaleDateString('es-ES', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' }) : 'RECIENTE'}
                                        </div>
                                    </div>
                                </motion.div>
                            ))}
                        </AnimatePresence>
                    </div>
                </div>

                <AnimatePresence>
                    {selectedFile && (
                        <motion.div
                            initial={{ x: 300, opacity: 0 }}
                            animate={{ x: 0, opacity: 1 }}
                            exit={{ x: 300, opacity: 0 }}
                            className="w-1/3 bg-black/40 border border-white/10 rounded-[2.5rem] flex flex-col overflow-hidden"
                        >
                            <header className="p-6 border-b border-white/5 flex items-center justify-between">
                                <div className="flex items-center gap-3">
                                    <FileText size={16} className="text-sentinel-blue" />
                                    <h2 className="text-[10px] font-black uppercase tracking-widest text-white/60">{selectedFile.name}</h2>
                                </div>
                                <div className="flex items-center gap-2">
                                    {!isEditing ? (
                                        <button
                                            onClick={() => setIsEditing(true)}
                                            className="px-3 py-1.5 bg-sentinel-blue/10 hover:bg-sentinel-blue/20 rounded-lg text-[9px] font-black uppercase transition-all"
                                        >
                                            Editar
                                        </button>
                                    ) : (
                                        <>
                                            <button
                                                onClick={async () => {
                                                    try {
                                                        await invoke('guardar_contenido_archivo_sentinel_media', { path: selectedFile.path, content: fileContent });
                                                        showToast('ok', 'Archivo guardado');
                                                        setIsEditing(false);
                                                        fetchFiles();
                                                    } catch (e) {
                                                        showToast('error', String(e));
                                                    }
                                                }}
                                                className="px-3 py-1.5 bg-sentinel-green/20 hover:bg-sentinel-green/30 text-sentinel-green rounded-lg text-[9px] font-black uppercase transition-all"
                                            >
                                                Guardar
                                            </button>
                                            <button
                                                onClick={() => setIsEditing(false)}
                                                className="px-3 py-1.5 bg-white/5 hover:bg-white/10 rounded-lg text-[9px] font-black uppercase transition-all"
                                            >
                                                Cancelar
                                            </button>
                                        </>
                                    )}
                                    <button
                                        onClick={() => {
                                            setSelectedFile(null);
                                            setIsEditing(false);
                                        }}
                                        className="p-2 hover:bg-white/5 rounded-xl text-white/20 hover:text-white transition-all"
                                    >
                                        <ArrowUpRight size={14} className="rotate-45" />
                                    </button>
                                </div>
                            </header>
                            <div className="p-6 border-b border-white/5 flex gap-2">
                                <button
                                    onClick={async () => {
                                        try {
                                            await invoke('analizar_archivo', { path: selectedFile.path });
                                            showToast('ok', 'Análisis iniciado correctamente');
                                        } catch (e) {
                                            console.error('Error al iniciar análisis:', e);
                                        }
                                    }}
                                    className="flex-1 py-2 px-4 bg-sentinel-blue/10 hover:bg-sentinel-blue/20 rounded-xl text-xs font-bold uppercase transition-all"
                                >
                                    Analizar
                                </button>
                                <button
                                    onClick={async () => {
                                        try {
                                            await invoke('traducir_archivo', { path: selectedFile.path });
                                            showToast('ok', 'Traducción iniciada correctamente');
                                        } catch (e) {
                                            console.error('Error al iniciar traducción:', e);
                                        }
                                    }}
                                    className="flex-1 py-2 px-4 bg-sentinel-green/10 hover:bg-sentinel-green/20 rounded-xl text-xs font-bold uppercase transition-all"
                                >
                                    Traducir
                                </button>
                                <button
                                    onClick={async () => {
                                        try {
                                            await invoke('ingestar_memoria', { path: selectedFile.path });
                                            showToast('ok', 'Ingestión a memoria neuronal iniciada');
                                        } catch (e) {
                                            console.error('Error al iniciar ingestión:', e);
                                        }
                                    }}
                                    className="flex-1 py-2 px-4 bg-purple-500/10 hover:bg-purple-500/20 rounded-xl text-xs font-bold uppercase transition-all"
                                >
                                    Ingestar
                                </button>
                            </div>
                            <div className="flex-1 overflow-hidden p-8 flex flex-col">
                                {isEditing ? (
                                    <textarea
                                        value={fileContent}
                                        onChange={(e) => setFileContent(e.target.value)}
                                        className="flex-1 w-full bg-transparent border-none outline-none text-[12px] font-mono text-white/60 leading-relaxed resize-none custom-scrollbar"
                                        spellCheck={false}
                                    />
                                ) : (
                                    <div className="flex-1 overflow-y-auto custom-scrollbar">
                                        <pre className="text-[11px] font-mono text-white/40 whitespace-pre-wrap leading-relaxed">
                                            {fileContent}
                                        </pre>
                                    </div>
                                )}
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>

            <footer className="p-6 rounded-[2rem] bg-white/[0.02] border border-white/5 flex items-center justify-between">
                <div className="flex items-center gap-6">
                    <span className="text-[10px] font-black text-white/30 uppercase tracking-widest">
                        {files.length} Unidades Sincronizadas
                    </span>
                </div>
                <button
                    onClick={async () => {
                        try {
                            await invoke('refrescar_indice_sentinel_media');
                            showToast('ok', 'Índice refrescado correctamente');
                        } catch (e) {
                            console.error('Error refrescando índice:', e);
                        }
                    }}
                    className="flex items-center gap-3 px-6 py-3 rounded-2xl bg-sentinel-blue/10 border border-sentinel-blue/20 text-sentinel-blue font-black text-[10px] uppercase tracking-widest hover:bg-sentinel-blue hover:text-cyber-dark transition-all"
                >
                    <Package size={14} /> Refrescar Índice
                </button>
                <button
                    onClick={async () => {
                        try {
                            await invoke('escanear_sentinel_media_fabrica');
                            showToast('ok', 'Escaneo de bóveda iniciado');
                            fetchFiles();
                        } catch (e) {
                            console.error('Error escaneando bóveda:', e);
                        }
                    }}
                    className="flex items-center gap-3 px-6 py-3 rounded-2xl bg-sentinel-green/10 border border-sentinel-green/20 text-sentinel-green font-black text-[10px] uppercase tracking-widest hover:bg-sentinel-green hover:text-cyber-dark transition-all"
                >
                    <Search size={14} /> Escanear Bóveda
                </button>
            </footer>
        </div>
    );
};

export default VaultView;
