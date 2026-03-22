import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import {
    TrendingUp,
    DollarSign,
    ArrowUpRight,
    BarChart3,
    RefreshCw,
    Target,
    Zap,
    PieChart
} from 'lucide-react';

interface CostSummary {
    total_today: number;
    total_this_month: number;
    total_all_time: number;
    total_revenue_today: number;
    total_revenue_this_month: number;
    total_revenue_all_time: number;
    global_roi_index: number;
    daily_budget_usage_pct: number;
    monthly_budget_usage_pct: number;
    by_provider: Record<string, {
        today: number;
        avg_cost_per_request: number;
        avg_efficiency_score: number;
        hardware_overhead_factor: number;
    }>;
    active_assets_count: number;
    smart_advice: string[];
}

interface ApiCall {
    timestamp: string;
    provider: string;
    model: string;
    cost_usd: number;
    success: boolean;
}

interface ProviderDetails {
    total_calls: number;
    successful_calls: number;
    failed_calls: number;
    total_cost: number;
    avg_cost_per_call: number;
}

interface CostProjection {
    projected_daily: number;
    projected_monthly: number;
    trend: string;
}

const SentinomicsView: React.FC = () => {
    const [data, setData] = useState<CostSummary | null>(null);
    const [loading, setLoading] = useState(false);
    const [gpuStatus, setGpuStatus] = useState<string>('Checking...');
    const [costProjection, setCostProjection] = useState<CostProjection | null>(null);

    // Estados de modales
    const [showProviderModal, setShowProviderModal] = useState(false);
    const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
    const [providerDetails, setProviderDetails] = useState<ProviderDetails | null>(null);

    const [showRecentCallsModal, setShowRecentCallsModal] = useState(false);
    const [recentCalls, setRecentCalls] = useState<ApiCall[]>([]);

    const [showBudgetModal, setShowBudgetModal] = useState(false);
    const [dailyBudget, setDailyBudget] = useState(10);
    const [monthlyBudget, setMonthlyBudget] = useState(300);
    const [alertThreshold, setAlertThreshold] = useState(80);

    const fetchData = async () => {
        setLoading(true);
        try {
            const result = await invoke<CostSummary>('get_resumen_costos');
            setData(result);

            // Obtener estado de GPU
            try {
                const gpu = await invoke<string>('check_gpu_status');
                setGpuStatus(gpu);
            } catch {
                setGpuStatus('GPU Not Available');
            }

            // Obtener proyeccion de costos
            try {
                const projection = await invoke<CostProjection>('get_cost_projection');
                setCostProjection(projection);
            } catch {
                console.error('Fallo al obtener proyeccion de costos');
            }
        } catch (e) {
            console.error('Error obteniendo datos de sentinomica:', e);
        } finally {
            setLoading(false);
        }
    };

    const handleProviderClick = async (provider: string) => {
        setSelectedProvider(provider);
        try {
            const details = await invoke<ProviderDetails>('get_detalles_proveedor', { provider });
            setProviderDetails(details);
            setShowProviderModal(true);
        } catch (e) {
            console.error('Error obteniendo detalles de proveedor:', e);
        }
    };

    const handleShowRecentCalls = async () => {
        try {
            const calls = await invoke<ApiCall[]>('get_llamadas_api_recientes', { provider: null, limit: 50 });
            setRecentCalls(calls);
            setShowRecentCallsModal(true);
        } catch (e) {
            console.error('Error obteniendo llamadas recientes:', e);
        }
    };

    const handleSaveBudget = async () => {
        try {
            await invoke('establecer_presupuesto', {
                daily: dailyBudget, // Se asume que dailyBudget corresponde a budgets.daily
                monthly: monthlyBudget, // Se asume que monthlyBudget corresponde a budgets.monthly
                threshold: alertThreshold // Se asume que alertThreshold corresponde a budsThreshold
            });
            setShowBudgetModal(false);
            fetchData(); // Refrescar datos
        } catch (e) {
            console.error('Error guardando presupuesto:', e);
        }
    };

    useEffect(() => {
        fetchData();
        const interval = setInterval(fetchData, 60000); // Sincronizacion 1 min
        return () => clearInterval(interval);
    }, []);

    if (!data) return (
        <div className="h-full flex items-center justify-center bg-cyber-dark text-sentinel-blue/20">
            <RefreshCw size={48} className="animate-spin" />
        </div>
    );

    const formatUSD = (val: number) => `$${val.toFixed(2)}`;
    const formatROI = (val: number) => `${val.toFixed(2)}x`;

    return (
        <div className="p-8 h-full overflow-y-auto flex flex-col bg-cyber-dark text-white selection:bg-sentinel-blue/30 antialiased custom-scrollbar">
            <header className="flex justify-between items-center mb-10 shrink-0">
                <div className="space-y-1">
                    <div className="flex items-center gap-3">
                        <TrendingUp className="text-sentinel-green" size={24} />
                        <h1 className="text-2xl font-black tracking-tighter uppercase">SENTI<span className="text-white/20">NOMICS</span></h1>
                    </div>
                    <p className="text-[10px] font-bold text-white/30 tracking-[0.4em] uppercase font-mono">Economic Intelligence // ROI Monitoring // Swarm Profitability</p>
                </div>

                <div className="flex items-center gap-4">
                    <div className="text-right mr-4">
                        <p className="text-[8px] font-black text-white/20 uppercase tracking-widest mb-1">Health Index</p>
                        <div className="flex items-center gap-2">
                            <div className="h-1 w-24 bg-white/5 rounded-full overflow-hidden">
                                <motion.div
                                    initial={{ width: 0 }}
                                    animate={{ width: `${Math.min(data.global_roi_index * 20, 100)}%` }}
                                    className="h-full bg-sentinel-green shadow-[0_0_10px_rgba(50,255,126,0.3)]"
                                />
                            </div>
                            <span className="text-[10px] font-mono text-sentinel-green font-bold">READY</span>
                        </div>
                    </div>
                    <button
                        onClick={fetchData}
                        className="p-3 rounded-xl bg-white/5 border border-white/5 hover:bg-white/10 transition-all text-white/40 hover:text-sentinel-green"
                    >
                        <RefreshCw size={18} className={loading ? "animate-spin" : ""} />
                    </button>
                </div>
            </header>

            <div className="grid grid-cols-12 gap-8">
                {/* ROI Dashboard */}
                <div className="col-span-12 grid grid-cols-4 gap-6">
                    <div className="p-8 rounded-[2rem] bg-sentinel-green/5 border border-sentinel-green/20 relative overflow-hidden group hover:border-sentinel-green/40 transition-all">
                        <div className="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
                            <Target size={64} />
                        </div>
                        <p className="text-[10px] font-black text-sentinel-green uppercase tracking-widest mb-2">Global ROI</p>
                        <h2 className="text-5xl font-black tracking-tighter text-white mb-2">{formatROI(data.global_roi_index)}</h2>
                        <div className="flex items-center gap-2 text-sentinel-green/60 text-[10px] font-bold uppercase">
                            <ArrowUpRight size={14} /> Efficiency Rating: {data.global_roi_index > 2 ? 'Optimized' : data.global_roi_index > 1 ? 'Good' : 'Needs Improvement'}
                        </div>
                    </div>

                    <div className="p-8 rounded-[2rem] bg-white/[0.03] border border-white/5 relative overflow-hidden">
                        <p className="text-[10px] font-black text-white/20 uppercase tracking-widest mb-2">Revenue Today</p>
                        <h2 className="text-4xl font-black tracking-tighter text-white mb-2">{formatUSD(data.total_revenue_today)}</h2>
                        <p className="text-[10px] font-mono text-white/40 uppercase tracking-widest">Active Assets: {data.active_assets_count}</p>
                    </div>

                    <div
                        onClick={handleShowRecentCalls}
                        className="p-8 rounded-[2rem] bg-white/[0.03] border border-white/5 relative overflow-hidden cursor-pointer hover:bg-white/[0.05] hover:border-white/10 transition-all"
                    >
                        <p className="text-[10px] font-black text-white/20 uppercase tracking-widest mb-2">Cost Today</p>
                        <h2 className="text-4xl font-black tracking-tighter text-white mb-2">{formatUSD(data.total_today)}</h2>
                        <div className="flex items-center gap-2 text-white/40 text-[10px] font-bold uppercase">
                            Burn rate: <span className={data.daily_budget_usage_pct > 80 ? 'text-red-500' : data.daily_budget_usage_pct > 50 ? 'text-yellow-500' : 'text-sentinel-blue'}>{data.daily_budget_usage_pct > 80 ? 'Critical' : data.daily_budget_usage_pct > 50 ? 'High' : 'Nominal'}</span>
                            {gpuStatus !== 'GPU Not Available' && <span className="ml-2 px-2 py-0.5 rounded-full bg-sentinel-blue/10 text-[8px] border border-sentinel-blue/20">{gpuStatus}</span>}
                        </div>
                    </div>

                    <div className="p-8 rounded-[2rem] bg-white/[0.03] border border-white/5 relative overflow-hidden">
                        <p className="text-[10px] font-black text-white/20 uppercase tracking-widest mb-2">Monthly Margin</p>
                        <h2 className="text-4xl font-black tracking-tighter text-sentinel-green mb-2">{formatUSD(data.total_revenue_this_month - data.total_this_month)}</h2>
                        <p className="text-[10px] font-mono text-white/40 uppercase tracking-widest">Net Gain Index</p>
                    </div>
                </div>

                {/* Progress Indicators */}
                <div className="col-span-8 space-y-8">
                    <div className="p-10 rounded-[2.5rem] bg-white/[0.02] border border-white/5 relative">
                        <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-[0.02] pointer-events-none" />
                        <h3 className="text-sm font-black uppercase tracking-widest mb-8 flex items-center gap-3">
                            <PieChart size={18} className="text-sentinel-blue" />
                            Distribución de Presupuestos
                        </h3>

                        <div className="space-y-8">
                            <div className="space-y-3">
                                <div className="flex justify-between items-end">
                                    <span className="text-[10px] font-bold text-white/40 uppercase tracking-widest">Sincronía Diaria (vs Burn)</span>
                                    <span className="text-xs font-mono font-black text-sentinel-blue">{data.daily_budget_usage_pct.toFixed(1)}%</span>
                                </div>
                                <div className="h-2 bg-white/5 rounded-full overflow-hidden">
                                    <motion.div
                                        initial={{ width: 0 }}
                                        animate={{ width: `${Math.min(data.daily_budget_usage_pct, 100)}%` }}
                                        className={`h-full ${data.daily_budget_usage_pct > 80 ? 'bg-yellow-500' : 'bg-sentinel-blue'}`}
                                    />
                                </div>
                            </div>

                            <div className="space-y-3">
                                <div className="flex justify-between items-end">
                                    <span className="text-[10px] font-bold text-white/40 uppercase tracking-widest">Sincronía Mensual (Goal Reach)</span>
                                    <span className="text-xs font-mono font-black text-sentinel-green">{data.monthly_budget_usage_pct.toFixed(1)}%</span>
                                </div>
                                <div className="h-2 bg-white/5 rounded-full overflow-hidden">
                                    <motion.div
                                        initial={{ width: 0 }}
                                        animate={{ width: `${Math.min(data.monthly_budget_usage_pct, 100)}%` }}
                                        className="h-full bg-sentinel-green"
                                    />
                                </div>
                            </div>

                            {costProjection && (
                                <div className="mt-6 p-4 bg-white/5 rounded-xl">
                                    <p className="text-[8px] text-white/40 uppercase mb-2">Projected Costs</p>
                                    <div className="flex justify-between text-xs">
                                        <span>Daily: {formatUSD(costProjection.projected_daily)}</span>
                                        <span>Monthly: {formatUSD(costProjection.projected_monthly)}</span>
                                    </div>
                                    <p className="text-[8px] text-white/60 mt-1">Trend: {costProjection.trend}</p>
                                </div>
                            )}

                            <button
                                onClick={() => setShowBudgetModal(true)}
                                className="mt-4 w-full py-2 bg-sentinel-blue/10 hover:bg-sentinel-blue/20 rounded-xl text-xs font-bold uppercase transition-all"
                            >
                                Set Budget
                            </button>
                        </div>
                    </div>

                    <div className="p-10 rounded-[2.5rem] bg-white/[0.02] border border-white/5">
                        <h3 className="text-sm font-black uppercase tracking-widest mb-6">Efficiency per Provider</h3>
                        <div className="grid grid-cols-2 gap-8">
                            {Object.entries(data.by_provider).map(([name, stats], i) => (
                                <div
                                    key={i}
                                    onClick={() => handleProviderClick(name)}
                                    className="flex flex-col gap-2 p-6 rounded-2xl bg-white/5 border border-white/5 relative overflow-hidden group cursor-pointer hover:bg-white/10 hover:border-white/10 transition-all"
                                >
                                    <div className="absolute top-0 right-0 p-4 opacity-5 group-hover:opacity-10 transition-opacity">
                                        <Zap size={32} />
                                    </div>
                                    <p className="text-[10px] font-black text-white uppercase tracking-widest">{name}</p>
                                    <div className="flex justify-between items-baseline mt-2">
                                        <span className="text-[8px] font-mono text-white/40">Efficiency Score:</span>
                                        <span className={`text-[10px] font-mono font-bold ${stats.avg_efficiency_score > 10 ? 'text-sentinel-green' : 'text-yellow-500'}`}>
                                            {stats.avg_efficiency_score.toFixed(2)}
                                        </span>
                                    </div>
                                    <div className="flex justify-between items-baseline">
                                        <span className="text-[8px] font-mono text-white/40">Hardware Overhead:</span>
                                        <span className="text-[8px] font-mono text-sentinel-blue">
                                            {stats.hardware_overhead_factor.toFixed(4)}
                                        </span>
                                    </div>
                                    <div className="mt-2 h-1 bg-white/5 rounded-full overflow-hidden">
                                        <motion.div
                                            initial={{ width: 0 }}
                                            animate={{ width: `${Math.min(stats.avg_efficiency_score * 5, 100)}%` }}
                                            className="h-full bg-sentinel-green"
                                        />
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>

                {/* Right Column: Insights */}
                <div className="col-span-4 space-y-8">
                    <div className="p-8 rounded-[2.5rem] bg-sentinel-blue/5 border border-sentinel-blue/20">
                        <h3 className="text-[10px] font-black uppercase tracking-widest mb-6 flex items-center gap-2">
                            <BarChart3 size={14} className="text-sentinel-blue" />
                            Smart Advice
                        </h3>
                        <div className="space-y-4">
                            {data.smart_advice.length > 0 ? (
                                data.smart_advice.map((advice, i) => (
                                    <div key={i} className="p-4 rounded-xl bg-white/5 border border-white/5 text-[10px] leading-relaxed text-white/60">
                                        <span className="text-sentinel-blue font-bold">INSIGHT:</span> {advice}
                                    </div>
                                ))
                            ) : (
                                <div className="p-4 rounded-xl bg-white/5 border border-white/5 text-[10px] leading-relaxed text-white/60">
                                    No hay avisos inteligentes disponibles en este momento.
                                </div>
                            )}
                        </div>
                    </div>

                    <div className="p-8 rounded-[2.5rem] bg-white/[0.03] border border-white/5 flex flex-col items-center justify-center text-center gap-4 py-12">
                        <div className="w-16 h-16 rounded-full bg-white/5 flex items-center justify-center border border-white/10">
                            <DollarSign size={24} className="text-sentinel-green animate-pulse" />
                        </div>
                        <div>
                            <p className="text-[10px] font-black uppercase tracking-widest text-white/60">Net Asset Worth</p>
                            <h4 className="text-2xl font-black font-mono mt-1 text-white">{formatUSD(data.total_revenue_all_time)}</h4>
                        </div>
                    </div>
                </div>
            </div>

            {/* Provider Details Modal */}
            {showProviderModal && providerDetails && (
                <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50" onClick={() => setShowProviderModal(false)}>
                    <div className="bg-cyber-dark border border-white/10 rounded-2xl p-8 max-w-2xl w-full m-4" onClick={(e) => e.stopPropagation()}>
                        <h3 className="text-2xl font-black mb-6 uppercase">{selectedProvider} Details</h3>
                        <div className="grid grid-cols-2 gap-4 mb-6">
                            <div className="p-4 bg-white/5 rounded-xl">
                                <p className="text-xs text-white/40 mb-1">Total Calls</p>
                                <p className="text-2xl font-bold">{providerDetails.total_calls}</p>
                            </div>
                            <div className="p-4 bg-white/5 rounded-xl">
                                <p className="text-xs text-white/40 mb-1">Success Rate</p>
                                <p className="text-2xl font-bold text-sentinel-green">
                                    {((providerDetails.successful_calls / providerDetails.total_calls) * 100).toFixed(1)}%
                                </p>
                            </div>
                            <div className="p-4 bg-white/5 rounded-xl">
                                <p className="text-xs text-white/40 mb-1">Total Cost</p>
                                <p className="text-2xl font-bold">{formatUSD(providerDetails.total_cost)}</p>
                            </div>
                            <div className="p-4 bg-white/5 rounded-xl">
                                <p className="text-xs text-white/40 mb-1">Avg Cost/Call</p>
                                <p className="text-2xl font-bold">{formatUSD(providerDetails.avg_cost_per_call)}</p>
                            </div>
                        </div>
                        <button
                            onClick={() => setShowProviderModal(false)}
                            className="w-full py-3 bg-sentinel-blue/20 hover:bg-sentinel-blue/30 rounded-xl font-bold transition-all"
                        >
                            Close
                        </button>
                    </div>
                </div>
            )}

            {/* Recent API Calls Modal */}
            {showRecentCallsModal && (
                <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50" onClick={() => setShowRecentCallsModal(false)}>
                    <div className="bg-cyber-dark border border-white/10 rounded-2xl p-8 max-w-4xl w-full m-4 max-h-[80vh] overflow-y-auto" onClick={(e) => e.stopPropagation()}>
                        <h3 className="text-2xl font-black mb-6 uppercase">Recent API Calls</h3>
                        <div className="space-y-2">
                            {recentCalls.map((call, idx) => (
                                <div key={idx} className="p-4 bg-white/5 rounded-xl flex justify-between items-center">
                                    <div>
                                        <p className="font-bold">{call.provider} - {call.model}</p>
                                        <p className="text-xs text-white/40">{new Date(call.timestamp).toLocaleString()}</p>
                                    </div>
                                    <div className="text-right">
                                        <p className="font-mono font-bold">{formatUSD(call.cost_usd)}</p>
                                        <p className={`text-xs ${call.success ? 'text-sentinel-green' : 'text-red-500'}`}>
                                            {call.success ? '✓ Success' : '✗ Failed'}
                                        </p>
                                    </div>
                                </div>
                            ))}
                        </div>
                        <button
                            onClick={() => setShowRecentCallsModal(false)}
                            className="w-full mt-6 py-3 bg-sentinel-blue/20 hover:bg-sentinel-blue/30 rounded-xl font-bold transition-all"
                        >
                            Close
                        </button>
                    </div>
                </div>
            )}

            {/* Budget Management Modal */}
            {showBudgetModal && (
                <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50" onClick={() => setShowBudgetModal(false)}>
                    <div className="bg-cyber-dark border border-white/10 rounded-2xl p-8 max-w-md w-full m-4" onClick={(e) => e.stopPropagation()}>
                        <h3 className="text-2xl font-black mb-6 uppercase">Set Budget</h3>
                        <div className="space-y-4">
                            <div>
                                <label className="text-xs text-white/60 mb-2 block">Daily Budget (USD)</label>
                                <input
                                    type="number"
                                    value={dailyBudget}
                                    onChange={(e) => setDailyBudget(parseFloat(e.target.value))}
                                    className="w-full p-3 bg-white/5 border border-white/10 rounded-xl text-white"
                                />
                            </div>
                            <div>
                                <label className="text-xs text-white/60 mb-2 block">Monthly Budget (USD)</label>
                                <input
                                    type="number"
                                    value={monthlyBudget}
                                    onChange={(e) => setMonthlyBudget(parseFloat(e.target.value))}
                                    className="w-full p-3 bg-white/5 border border-white/10 rounded-xl text-white"
                                />
                            </div>
                            <div>
                                <label className="text-xs text-white/60 mb-2 block">Alert Threshold (%)</label>
                                <input
                                    type="number"
                                    value={alertThreshold}
                                    onChange={(e) => setAlertThreshold(parseFloat(e.target.value))}
                                    className="w-full p-3 bg-white/5 border border-white/10 rounded-xl text-white"
                                />
                            </div>
                        </div>
                        <div className="flex gap-4 mt-6">
                            <button
                                onClick={() => setShowBudgetModal(false)}
                                className="flex-1 py-3 bg-white/5 hover:bg-white/10 rounded-xl font-bold transition-all"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleSaveBudget}
                                className="flex-1 py-3 bg-sentinel-green/20 hover:bg-sentinel-green/30 rounded-xl font-bold transition-all"
                            >
                                Save
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};

export default SentinomicsView;
