import React, { useState, useMemo } from 'react';
import { Terminal, Search, X } from 'lucide-react';
import { ProcessedLog } from '../../hooks/useDashboardData';
import { LogDetailModal } from './LogDetailModal';

interface SystemLogsWidgetProps {
    logs: ProcessedLog[];
}

export const SystemLogsWidget: React.FC<SystemLogsWidgetProps> = ({ logs }) => {
    const [selectedLog, setSelectedLog] = useState<ProcessedLog | null>(null);
    const [searchQuery, setSearchQuery] = useState('');
    const [severityFilter, setSeverityFilter] = useState<string | null>(null);

    // Filtrar logs
    const filteredLogs = useMemo(() => {
        return logs.filter(log => {
            const matchesSearch = searchQuery === '' ||
                log.message.toLowerCase().includes(searchQuery.toLowerCase());
            const matchesSeverity = severityFilter === null ||
                log.severity === severityFilter;
            return matchesSearch && matchesSeverity;
        });
    }, [logs, searchQuery, severityFilter]);

    const severities = ['Critical', 'Warning', 'HardwareAlert', 'Info'];
    const severityCounts = useMemo(() => {
        return severities.reduce((acc, severity) => {
            acc[severity] = logs.filter(log => log.severity === severity).length;
            return acc;
        }, {} as Record<string, number>);
    }, [logs]);

    return (
        <>
            <div className="p-6 rounded-[2rem] bg-black/20 border border-white/5">
                {/* Header */}
                <div className="flex justify-between items-center mb-4">
                    <h3 className="text-[10px] font-black text-white/40 uppercase tracking-[0.3em] flex items-center gap-2">
                        <Terminal size={12} /> System Logs
                    </h3>
                    <div className="flex items-center gap-2">
                        <span className="w-2 h-2 rounded-full bg-sentinel-green animate-pulse" />
                        <span className="text-[10px] font-bold text-sentinel-green uppercase tracking-wider">Live</span>
                    </div>
                </div>

                {/* Search Bar */}
                <div className="mb-4">
                    <div className="relative">
                        <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-white/30" />
                        <input
                            type="text"
                            placeholder="Search logs..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="w-full pl-10 pr-10 py-2 rounded-xl bg-white/5 border border-white/10 text-white text-xs placeholder:text-white/30 focus:outline-none focus:border-sentinel-blue/50 transition-colors"
                        />
                        {searchQuery && (
                            <button
                                onClick={() => setSearchQuery('')}
                                className="absolute right-3 top-1/2 -translate-y-1/2 text-white/30 hover:text-white/60 transition-colors"
                            >
                                <X size={14} />
                            </button>
                        )}
                    </div>
                </div>

                {/* Severity Filters */}
                <div className="flex gap-2 mb-4 flex-wrap">
                    <button
                        onClick={() => setSeverityFilter(null)}
                        className={`px-3 py-1.5 rounded-lg text-[9px] font-black uppercase tracking-wider transition-all ${severityFilter === null
                            ? 'bg-sentinel-blue/20 text-sentinel-blue border border-sentinel-blue/30'
                            : 'bg-white/5 text-white/40 border border-white/10 hover:bg-white/10'
                            }`}
                    >
                        All ({logs.length})
                    </button>
                    {severities.map(severity => (
                        <button
                            key={severity}
                            onClick={() => setSeverityFilter(severity)}
                            className={`px-3 py-1.5 rounded-lg text-[9px] font-black uppercase tracking-wider transition-all ${severityFilter === severity
                                ? severity === 'Critical' ? 'bg-red-500/20 text-red-500 border border-red-500/30'
                                    : severity === 'Warning' ? 'bg-yellow-500/20 text-yellow-500 border border-yellow-500/30'
                                        : severity === 'HardwareAlert' ? 'bg-cyan-500/20 text-cyan-500 border border-cyan-500/30'
                                            : 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
                                : 'bg-white/5 text-white/40 border border-white/10 hover:bg-white/10'
                                }`}
                        >
                            {severity} ({severityCounts[severity] || 0})
                        </button>
                    ))}
                </div>

                {/* Logs List */}
                <div className="space-y-2 font-mono text-[9px] max-h-64 overflow-y-auto custom-scrollbar">
                    {filteredLogs.length === 0 ? (
                        <div className="text-white/20 italic text-center py-8">
                            {searchQuery || severityFilter
                                ? 'No logs match your filters'
                                : 'Esperando eventos críticos...'
                            }
                        </div>
                    ) : (
                        filteredLogs.map((log, i) => (
                            <button
                                key={i}
                                onClick={() => setSelectedLog(log)}
                                className="w-full flex gap-2 items-center p-2 rounded-lg hover:bg-white/5 transition-colors cursor-pointer group"
                            >
                                <span className="text-white/20 shrink-0">
                                    {new Date(log.timestamp * 1000).toLocaleTimeString([], { hour12: false })}
                                </span>
                                <span className={`px-1.5 py-0.5 rounded-[0.2rem] font-bold tracking-wider shrink-0 ${log.severity === 'Critical' ? 'bg-red-500/20 text-red-500' :
                                    log.severity === 'Warning' ? 'bg-yellow-500/20 text-yellow-500' :
                                        log.severity === 'HardwareAlert' ? 'bg-cyan-500/20 text-cyan-500' :
                                            'bg-blue-500/10 text-blue-400'
                                    }`}>
                                    {(log.severity || 'Info').toUpperCase()}
                                </span>
                                <span className="text-white/60 truncate flex-1 text-left group-hover:text-white/80 transition-colors">
                                    {log.message}
                                </span>
                                <span className="text-white/20 text-[8px] opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                                    Click for details
                                </span>
                            </button>
                        ))
                    )}
                </div>
            </div>

            {/* Modal */}
            {selectedLog && (
                <LogDetailModal
                    log={selectedLog}
                    onClose={() => setSelectedLog(null)}
                />
            )}
        </>
    );
};
