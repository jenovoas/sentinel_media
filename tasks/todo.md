# Plan de Tareas - Sentinel Media

## Contexto y Análisis (Issue)

Se produjo una alucinación relacionada a las rutas del sistema, asumiendo una ruta absoluta para la bóveda (`/home/jnovoas/Desarrollo/obsidian`) que en realidad estaba anidada dentro del espacio de trabajo (`/home/jnovoas/Desarrollo/sentinel_media/obsidian`).
El usuario me corrigió adjuntando imagen de su explorador comprobando `sentinel_media/obsidian`. Tras verificarlo, se renombró correctamente a `sentinel_media/vault` y se eliminó la basura de `_Agentes` allí contenida.
Ahorora debemos cerciorarnos de que cualquier referencia en el código a `/home/jnovoas/Desarrollo/obsidian` se remueva por completo y se use resolución relativa (por convención `vault/` local o argumento). Las reglas del usuario prohíben arreglos temporales, necesitamos refactorizar el cause raíz.

## Acciones (Todos)

- [x] 1. Identificar en `sentinel_research` y `sentinel_scanner` el rastreo de cualquier string absoluto hacia `obsidian` usando `grep`.
- [x] 2. Seguir la spec de "Pipeline de Generación de Guiones - Plan de Implementación" mostrada en el screenshot del usuario (archivo: `vault/docs/superpowers/plans/pipeline-guion.md`), Chunk 1: Tarea 1.
- [x] 3. Implementar el Helper `sentinel_media_home()` o `sentinel_home()` sugerido en la SPEC para normalizar la ruta base a través del sistema.
- [x] 4. Aplicar los cambios y remover los harcodeos residuales en el research, core_agent y en la GUI Tauri.
- [x] 5. Probar con `cargo check --workspace` resolviendo la dependencia a nivel global.
- [x] 6. Actualizar `tasks/todo.md` reportando revisión.
- [ ] 7. Informar exitosamente al usuario y demostrarlo funcionando.
