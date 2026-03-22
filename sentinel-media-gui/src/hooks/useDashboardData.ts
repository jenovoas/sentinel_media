import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// === Senior Engineering Patterns: Robust Frontend Types ===

export interface OpEntry {
    id: string;
    status: string;
    target_file: string;
    updated_at?: string;
    op_type: string;
    engine: string;
    progress_pct: number;
}

export interface ClaimStatus {
    name: string;
    status: string;
    active: boolean;
    value: string;
    icon_type: string;
}

export interface CortexStats {
    cpu_usage: number;
    memory_used: number;
    memory_total: number;
    uptime: number;
    firewall_active: boolean;
    logs_total: number;
    kernel_version: string;
    claims: ClaimStatus[];
    cpu_temp: number;
    coherence: number;
    scheduler_efficiency: number;
    truthsync_audit?: {
        timestamp: string;
        global_integrity: number;
        switches: Record<string, any>;
    };
}

// Robust Hardware Status matching Rust Enum
export type HardwareStatus =
    | { status: 'Active'; data: { temp: number; usage: number; memory: string; fan_speed?: number } }
    | { status: 'Throttling'; data: { temp: number; reason: string } }
    | { status: 'Offline'; data: { last_seen: string; error?: string } };

// Enum matching Rust LogSeverity
export type LogSeverity = 'Info' | 'Warning' | 'Critical' | 'HardwareAlert';

export interface ProcessedLog {
    message: string;
    severity: LogSeverity;
    timestamp: number; // Unix timestamp number
}

export interface DashboardData {
    operations: OpEntry[];
    stats: CortexStats | null;
    systemLogs: ProcessedLog[];
    gpuStatus: HardwareStatus | null;
    cpuTemp: number;
    activeAgents: any[];
    factoryStatus: { running: boolean, pid: number | null };
}

export type DashboardState =
    | { status: 'idle' }
    | { status: 'loading' }
    | { status: 'success', data: DashboardData }
    | { status: 'error', error: string };

export const useDashboardData = (pollInterval: number = 5000) => {
    const [state, setState] = useState<DashboardState>({ status: 'loading' });

    const fetchData = useCallback(async () => {
        try {
            const opsPromise = invoke<OpEntry[]>('get_operaciones').catch(e => {
                console.error("❌ Error fetching Operations:", e);
                return [];
            });
            const logsPromise = invoke<ProcessedLog[]>('get_logs_sistema', { count: 50 }).catch(e => {
                console.error("❌ Error fetching Logs:", e);
                return [];
            });
            const gpuPromise = invoke<HardwareStatus>('check_gpu_status').catch(e => {
                console.error("❌ Error fetching GPU Status:", e);
                return null;
            });
            const agentsPromise = invoke<any[]>('get_agentes').catch(e => {
                console.error("❌ Error fetching Agents:", e);
                return [];
            });
            const statsPromise = invoke<CortexStats>('get_estadisticas_cortex').catch(e => {
                console.error("❌ Error fetching Cortex Stats:", e);
                return null;
            });
            const factoryStatusPromise = invoke<{ running: boolean, pid: number | null }>('get_factory_status').catch(e => {
                console.error("❌ Error fetching Factory Status:", e);
                return { running: false, pid: null };
            });

            const [ops, logs, gpu, agents, stats, factoryStatus] = await Promise.all([
                opsPromise, logsPromise, gpuPromise, agentsPromise, statsPromise, factoryStatusPromise
            ]);

            setState({
                status: 'success',
                data: {
                    operations: ops,
                    stats: stats,
                    systemLogs: logs,
                    gpuStatus: gpu,
                    cpuTemp: stats?.cpu_temp || 0,
                    activeAgents: agents,
                    factoryStatus: factoryStatus
                }
            });

        } catch (e) {
            console.error(e);
            setState({ status: 'error', error: String(e) });
        }
    }, []);

    useEffect(() => {
        fetchData(); // Initial fetch
        const interval = setInterval(fetchData, pollInterval);
        return () => clearInterval(interval);
    }, [pollInterval, fetchData]);

    return { state, refetch: fetchData };
};
