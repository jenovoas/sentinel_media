import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { ProcessedLog } from './useDashboardData';
import { isTauri } from '../utils/isTauri';

const MAX_LOGS = 50; // Buffer circular en UI para proteger memoria

/**
 * Hook reactivo para Logs de Sistema.
 * Escucha 'sys-log' y mantiene un buffer circular.
 */
export const useLogStream = (initialLogs: ProcessedLog[] = []) => {
    const [logs, setLogs] = useState<ProcessedLog[]>(initialLogs);
    const logsRef = useRef<ProcessedLog[]>(initialLogs); // Ref para acceso dentro del closure del listener

    useEffect(() => {
        if (initialLogs.length > 0) {
            setLogs(initialLogs);
            logsRef.current = initialLogs;
        }
    }, [initialLogs]);

    useEffect(() => {
        if (!isTauri()) return;

        const unlisten = listen<ProcessedLog>('sys-log', (event) => {
            const newLog = event.payload;

            // Actualización funcional con Buffer Circular
            setLogs(prevLogs => {
                const updated = [newLog, ...prevLogs].slice(0, MAX_LOGS);
                logsRef.current = updated;
                return updated;
            });
        });

        return () => {
            unlisten.then(f => f());
        };
    }, []);

    return logs;
};
