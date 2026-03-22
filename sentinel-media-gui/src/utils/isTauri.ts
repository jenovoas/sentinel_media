/**
 * Detecta si la aplicación se está ejecutando dentro del entorno Tauri.
 * Verifica la presencia de window.__TAURI_INTERNALS__ que Tauri inyecta.
 */
export const isTauri = (): boolean => {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};
