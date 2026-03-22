# Lecciones Aprendidas

1. **Rutas asume-ciego**: NUNCA asumir la existencia o ubicación de un directorio basándose en historia pasada. SIEMPRE usar herramientas de exploración (`ls`, `fd`) para confirmar la ubicación actual *antes* de ejecutar operaciones destructivas o de movimiento como `mv` o `rm`.
2. **Priorizar Reglas Globales**: Leer y aplicar estrictamente el perfil de reglas proporcionado por el usuario (Orquestación del Flujo de Trabajo). Todo paso no trivial requiere Planificación formal en `tasks/todo.md`.
