import React from 'react';
import { motion } from 'framer-motion';
import { Server, Cpu, Zap, AlertTriangle, PowerOff } from 'lucide-react';
import { HardwareStatus } from '../../hooks/useDashboardData';

interface HardwareMetricsProps {
    memoryUsed: number;
    memoryTotal: number;
    uptime: number;
    cpuTemp: string;
    gpuStatus: HardwareStatus | null;
}

export const HardwareMetrics: React.FC<HardwareMetricsProps> = ({
    memoryUsed,
    memoryTotal,
    uptime,
    cpuTemp,
    gpuStatus
}) => {
    const memoryUsedGB = (memoryUsed / 1024 / 1024 / 1024).toFixed(1);
    const memoryPercent = (memoryUsed / memoryTotal) * 100;
    const uptimeHours = Math.floor(uptime / 3600);
    const uptimeMinutes = Math.floor((uptime % 3600) / 60);

    // Helpers para GPU rendering basado en estado
    const renderGPUContent = () => {
        if (!gpuStatus) return null;

        switch (gpuStatus.status) {
            case 'Active': {
                const { temp, usage, memory } = gpuStatus.data;
                const isWarning = temp > 80;

                return (
                    <div className="space-y-4">
                        {/* Header Active */}
                        <div className="flex items-center justify-between">
                            <div className="text-[10px] font-bold text-sentinel-blue uppercase tracking-wider flex items-center gap-2">
                                <div className="w-2 h-2 rounded-full bg-sentinel-blue animate-pulse" />
                                ONLINE
                            </div>
                            <span className={`text-[12px] font-mono ${isWarning ? 'text-red-400' : 'text-sentinel-green'}`}>
                                {temp.toFixed(1)}°C
                            </span>
                        </div>

                        {/* Utilization Bar */}
                        <div className="space-y-2">
                            <div className="flex justify-between text-[10px] font-black text-white/30 uppercase tracking-widest">
                                <span>Utilización</span>
                                <span className="text-sentinel-blue">{usage.toFixed(1)}%</span>
                            </div>
                            <div className="h-1.5 w-full bg-white/5 rounded-full overflow-hidden">
                                <motion.div
                                    initial={{ width: 0 }}
                                    animate={{ width: `${usage}%` }}
                                    className={`h-full shadow-[0_0_10px_rgba(59,130,246,0.3)] ${usage > 90 ? 'bg-red-500' : 'bg-sentinel-blue'}`}
                                />
                            </div>
                        </div>

                        {/* Memory Text */}
                        <div className="p-3 rounded-xl bg-white/5 border border-white/5 flex justify-between items-center">
                            <span className="text-[10px] font-bold text-white/60 uppercase tracking-wider">VRAM USAGE</span>
                            <span className="text-[10px] font-mono text-white/40">{memory}</span>
                        </div>
                    </div>
                );
            }

            case 'Throttling': {
                const { temp, reason } = gpuStatus.data;
                return (
                    <div className="space-y-4">
                        <div className="flex items-center justify-between text-yellow-500">
                            <div className="text-[10px] font-bold uppercase tracking-wider flex items-center gap-2">
                                <AlertTriangle size={14} />
                                THROTTLING
                            </div>
                            <span className="text-[12px] font-mono">{temp.toFixed(1)}°C</span>
                        </div>
                        <div className="p-3 rounded-xl bg-yellow-500/10 border border-yellow-500/20 text-[10px] font-mono text-yellow-200">
                            ⚠️ {reason}
                        </div>
                    </div>
                );
            }

            case 'Offline': {
                return (
                    <div className="space-y-4 opacity-50">
                        <div className="flex items-center gap-2 text-white/40">
                            <PowerOff size={14} />
                            <span className="text-[10px] font-bold uppercase tracking-wider">OFFLINE / NO DETECTADO</span>
                        </div>
                        {gpuStatus.data.error && (
                            <div className="text-[10px] font-mono text-red-400/80 truncate">
                                Error: {gpuStatus.data.error}
                            </div>
                        )}
                    </div>
                );
            }
        }
    };

    return (
        <div className="space-y-6">
            {/* Métricas de Sistema */}
            <div className="p-8 rounded-[2.5rem] bg-gradient-to-br from-white/[0.03] to-transparent border border-white/10">
                <h3 className="text-xs font-black text-white/60 uppercase tracking-[0.4em] mb-8 flex items-center gap-3">
                    <Server size={16} className="text-sentinel-blue" />
                    MÉTRICAS FÍSICAS
                </h3>

                <div className="space-y-8">
                    {/* Memoria RAM */}
                    <div className="space-y-3">
                        <div className="flex justify-between text-[10px] font-black text-white/30 uppercase tracking-widest">
                            <span>Memoria RAM</span>
                            <span className="text-sentinel-green">{memoryUsedGB}GB</span>
                        </div>
                        <div className="h-1.5 w-full bg-white/5 rounded-full overflow-hidden">
                            <motion.div
                                initial={{ width: 0 }}
                                animate={{ width: `${memoryPercent}%` }}
                                className="h-full bg-sentinel-green shadow-[0_0_10px_rgba(16,185,129,0.3)]"
                            />
                        </div>
                    </div>

                    {/* Uptime */}
                    <div className="pt-4 p-5 rounded-2xl bg-white/5 border border-white/5 text-center">
                        <div className="text-[9px] font-black text-white/20 uppercase tracking-[0.3em] mb-3">
                            Tiempo de Actividad
                        </div>
                        <div className="text-xl font-black text-white/60 font-mono">
                            {uptimeHours}h {uptimeMinutes}m
                        </div>
                    </div>
                </div>
            </div>

            {/* CPU Temperature */}
            <div className="p-8 rounded-[2.5rem] glass-panel border border-white/5 text-center">
                <h3 className="text-xs font-black text-white/60 uppercase tracking-[0.4em] mb-4 flex items-center justify-center gap-2">
                    <Cpu size={16} className="text-sentinel-blue" />
                    Temp de CPU
                </h3>
                <div className="text-3xl font-black text-sentinel-green font-mono">
                    {cpuTemp}
                </div>
            </div>

            {/* GPU Status Block */}
            <div className={`p-8 rounded-[2.5rem] border transition-colors ${gpuStatus?.status === 'Active' ? 'bg-gradient-to-br from-sentinel-blue/5 to-transparent border-sentinel-blue/20' :
                    gpuStatus?.status === 'Throttling' ? 'bg-yellow-500/5 border-yellow-500/20' :
                        'bg-white/5 border-white/10'
                }`}>
                <h3 className={`text-xs font-black uppercase tracking-[0.4em] mb-6 flex items-center gap-3 ${gpuStatus?.status === 'Throttling' ? 'text-yellow-500' : 'text-sentinel-blue'
                    }`}>
                    <Zap size={16} />
                    GPU UNIT
                </h3>
                {renderGPUContent()}
            </div>
        </div>
    );
};
