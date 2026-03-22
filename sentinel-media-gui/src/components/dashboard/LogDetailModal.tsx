import React from 'react';
import { X, Copy, AlertTriangle, Info, Zap } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface LogDetailModalProps {
    log: {
        timestamp: number;
        severity: string;
        message: string;
        stackTrace?: string;
        context?: {
            cpu?: number;
            memory?: number;
            process?: string;
        };
    } | null;
    onClose: () => void;
}

export const LogDetailModal: React.FC<LogDetailModalProps> = ({ log, onClose }) => {
    if (!log) return null;

    const getSeverityIcon = () => {
        switch (log.severity) {
            case 'Critical': return <AlertTriangle className="text-red-500" size={24} />;
            case 'Warning': return <Zap className="text-yellow-500" size={24} />;
            case 'HardwareAlert': return <Zap className="text-cyan-500" size={24} />;
            default: return <Info className="text-blue-400" size={24} />;
        }
    };

    const getSeverityColor = () => {
        switch (log.severity) {
            case 'Critical': return 'border-red-500/30 bg-red-500/5';
            case 'Warning': return 'border-yellow-500/30 bg-yellow-500/5';
            case 'HardwareAlert': return 'border-cyan-500/30 bg-cyan-500/5';
            default: return 'border-blue-500/30 bg-blue-500/5';
        }
    };

    const copyToClipboard = () => {
        const logText = `[${new Date(log.timestamp * 1000).toISOString()}] ${log.severity}: ${log.message}${log.stackTrace ? '\n\nStack Trace:\n' + log.stackTrace : ''}`;
        navigator.clipboard.writeText(logText);
    };

    return (
        <AnimatePresence>
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4"
                onClick={onClose}
            >
                <motion.div
                    initial={{ scale: 0.9, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    exit={{ scale: 0.9, opacity: 0 }}
                    transition={{ type: "spring", damping: 25, stiffness: 300 }}
                    className={`max-w-3xl w-full rounded-[2rem] border ${getSeverityColor()} p-8 relative`}
                    onClick={(e) => e.stopPropagation()}
                >
                    {/* Close Button */}
                    <button
                        onClick={onClose}
                        className="absolute top-6 right-6 p-2 rounded-xl bg-white/5 hover:bg-white/10 transition-colors"
                    >
                        <X size={20} className="text-white/60" />
                    </button>

                    {/* Header */}
                    <div className="flex items-start gap-4 mb-6">
                        {getSeverityIcon()}
                        <div className="flex-1">
                            <div className="flex items-center gap-3 mb-2">
                                <h2 className="text-xl font-black uppercase tracking-tight text-white">
                                    {log.severity} Event
                                </h2>
                                <span className="text-xs font-mono text-white/40">
                                    {new Date(log.timestamp * 1000).toLocaleString()}
                                </span>
                            </div>
                            <p className="text-sm text-white/60 font-mono">
                                Timestamp: {log.timestamp}
                            </p>
                        </div>
                    </div>

                    {/* Message */}
                    <div className="mb-6">
                        <h3 className="text-xs font-black uppercase tracking-widest text-white/40 mb-3">
                            Message
                        </h3>
                        <div className="p-4 rounded-xl bg-black/30 border border-white/5">
                            <p className="text-sm text-white/80 font-mono leading-relaxed">
                                {log.message}
                            </p>
                        </div>
                    </div>

                    {/* Stack Trace */}
                    {log.stackTrace && (
                        <div className="mb-6">
                            <h3 className="text-xs font-black uppercase tracking-widest text-white/40 mb-3">
                                Stack Trace
                            </h3>
                            <div className="p-4 rounded-xl bg-black/30 border border-white/5 max-h-48 overflow-y-auto custom-scrollbar">
                                <pre className="text-xs text-white/60 font-mono whitespace-pre-wrap">
                                    {log.stackTrace}
                                </pre>
                            </div>
                        </div>
                    )}

                    {/* System Context */}
                    {log.context && (
                        <div className="mb-6">
                            <h3 className="text-xs font-black uppercase tracking-widest text-white/40 mb-3">
                                System Context
                            </h3>
                            <div className="grid grid-cols-3 gap-4">
                                {log.context.cpu !== undefined && (
                                    <div className="p-3 rounded-xl bg-black/30 border border-white/5">
                                        <p className="text-[10px] font-bold text-white/40 uppercase tracking-wider mb-1">CPU</p>
                                        <p className="text-lg font-black text-white">{log.context.cpu.toFixed(1)}%</p>
                                    </div>
                                )}
                                {log.context.memory !== undefined && (
                                    <div className="p-3 rounded-xl bg-black/30 border border-white/5">
                                        <p className="text-[10px] font-bold text-white/40 uppercase tracking-wider mb-1">Memory</p>
                                        <p className="text-lg font-black text-white">{(log.context.memory / 1024 / 1024 / 1024).toFixed(1)} GB</p>
                                    </div>
                                )}
                                {log.context.process && (
                                    <div className="p-3 rounded-xl bg-black/30 border border-white/5">
                                        <p className="text-[10px] font-bold text-white/40 uppercase tracking-wider mb-1">Process</p>
                                        <p className="text-sm font-mono text-white truncate">{log.context.process}</p>
                                    </div>
                                )}
                            </div>
                        </div>
                    )}

                    {/* Actions */}
                    <div className="flex gap-3">
                        <button
                            onClick={copyToClipboard}
                            className="flex-1 flex items-center justify-center gap-2 px-6 py-3 rounded-xl bg-sentinel-blue/10 hover:bg-sentinel-blue/20 border border-sentinel-blue/30 text-sentinel-blue font-black uppercase tracking-widest text-xs transition-all"
                        >
                            <Copy size={14} />
                            Copy to Clipboard
                        </button>
                        <button
                            onClick={onClose}
                            className="px-6 py-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-white/60 font-black uppercase tracking-widest text-xs transition-all"
                        >
                            Close
                        </button>
                    </div>
                </motion.div>
            </motion.div>
        </AnimatePresence>
    );
};
