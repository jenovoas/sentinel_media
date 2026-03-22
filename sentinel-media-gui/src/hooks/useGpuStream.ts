import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { HardwareStatus } from './useDashboardData';

/**
 * Hook reactivo para métricas de GPU.
 * Escucha el evento 'gpu-metrics' emitido por el backend (NVML).
 * Mantiene el último estado conocido.
 */
export const useGpuStream = (initialStatus: HardwareStatus | null) => {
    const [status, setStatus] = useState<HardwareStatus | null>(initialStatus);

    useEffect(() => {
        // Actualizar estado inicial si cambia (ej. primera carga de useDashboardData)
        if (initialStatus) {
            setStatus(initialStatus);
        }
    }, [initialStatus]);

    useEffect(() => {
        // Suscripción al canal de eventos
        const unlisten = listen<HardwareStatus>('gpu-metrics', (event) => {
            console.log('🔥 GPU Update:', event.payload);
            setStatus(event.payload);
        });

        // Cleanup al desmontar
        return () => {
            unlisten.then(f => f());
        };
    }, []);

    return status;
};
