# Lecciones Aprendidas

1. **Rutas asume-ciego**: NUNCA asumir la existencia o ubicación de un directorio basándose en historia pasada. SIEMPRE usar herramientas de exploración (`ls`, `fd`) para confirmar la ubicación actual *antes* de ejecutar operaciones destructivas o de movimiento como `mv` o `rm`.
2. **Priorizar Reglas Globales**: Leer y aplicar estrictamente el perfil de reglas proporcionado por el usuario (Orquestación del Flujo de Trabajo). Todo paso no trivial requiere Planificación formal en `tasks/todo.md`.

3. **Cero Simplicación (MANDATO ABSOLUTO)**: Prohibido estrictamente borrar funciones de Rust, instalaciones apt-get, lógicas de pipeline CI/CD o abreviar código bajo la excusa de 'limpiar' y simplificar. Preservación Atómica de la arquitectura.
4. **Contexto Exclusivo de Proyecto**: Revisar qué repositorio se está alterando antes de restaurar o generar prompts. Sentinel Media es la Bóveda/YouTube Factory, no aplicar física ni base-60 matemática proveniente del Kernel ME-60OS.
