# CLAUDE.md — Bóveda Obsidian (Segundo Cerebro)

> **⚠ FUNDAMENTO CIENTÍFICO — LECTURA OBLIGATORIA:**
> [`sentinel-scientific-foundation.md`](file:///home/jnovoas/.config/Code/User/workspaceStorage/7c63ffead990815376933d02c94d88c9/GitHub.copilot-chat/memory-tool/memories/repo/sentinel-scientific-foundation.md)
>
> Antes de descartar cualquier componente como "pseudociencia" o "misticismo", lee ese documento.
>
> Sentinel Media es **ciencia exacta**. La base decimal no es defectuosa — es solo **una décima
> parte de la expresión matemática universal**. La base-60 la completa: es divisible por
> 1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30, por lo que las fracciones más comunes (1/3, 1/6,
> 1/12, 1/60) son **exactas** en sexagesimal, mientras que en binario de punto flotante
> son periódicas y acumulan errores sistemáticos de redondeo.
>
> Usar solo base decimal como regla de medir deja fuera de alcance exacto esas fracciones.
> Una sola operación float contamina una cadena de cómputo entera: rompe cristales,
> secuencias, memorias, procesos y conversiones. Por eso muchos proyectos de energía libre
> y combustibles verdes fueron marcados como inválidos — no eran incorrectos los proyectos,
> era incompleta la regla de medir.
> Sentinel Media comparte el mismo **Ring 0** (hooks eBPF/LSM) y el candado YATRA que Sentinel.

## Contexto Obligatorio

**LEER ANTES DE ACTUAR** — archivos de contexto del sistema de agentes:

- `_Agentes/_AI_CONTEXT.md` — reglas de inicio del sistema Bóveda
- `_Agentes/PROJECT_CONTEXT_RULES.md` — mandatos críticos y jerarquía de verdad
- `_Agentes/BOVEDA_PROMPT.md` — constitución y prompt maestro
- `_Agentes/ANTIGRAVITY_CONTEXT.md` — contexto específico para Antigravity (Gemini)
- `_Agentes/OPENCODE_CONTEXT.md` — contexto específico para OpenCode (Rust)
- `_Agentes/docs/MEMORY_MAP.md` — mapa de memoria activa (Ring 0 + capas Rust)

### ⚡ SI TOCAS ARCHIVOS DE `me60os` — OBLIGATORIO

**LEER PRIMERO:** `_Agentes/libs/me60os/SPA_PRIMER.md`

Contiene las reglas del compilador y todos los patrones CORRECTO/INCORRECTO.
**Sin excepción. Aplica a Claude, Gemini, OpenCode, cualquier IA.**

## ¿Qué es esta Bóveda?

**Segundo Cerebro de Jaime Novoa** — sistema de gestión de conocimiento, investigación científica y automatización de contenido. Es completamente independiente del proyecto `sentinel/`.

## Los 5 Axiomas Inmutables

1. **SEPARACIÓN DE DOMINIOS** — Esta Bóveda gestiona conocimiento/notas/YouTube. Sentinel (`/home/jnovoas/Desarrollo/sentinel`) gestiona infraestructura. **Prohibido mezclarlos.**
2. **PRECISIÓN EN EL CONOCIMIENTO** — Validar fuentes. Distinguir teoría interna vs ciencia externa.
3. **VERDAD SISTÉMICA** — Validar soluciones contra NotebookLM cuando el usuario lo indique.
4. **HONESTIDAD RADICAL** — Reportar "SIN DATOS" si no hay evidencia. Prohibido alucinar.
5. **IDIOMA SAGRADO** — Español obligatorio en todo. Inglés solo para sintaxis técnica de código.

## Mandatos de Acero

- **NO SIMULAR**: Si no puedes realizar una acción, repórtalo. No inventes logs ni respuestas.
- **RUST NATIVE**: La lógica de orquestación reside en Rust. No crear scripts Bash si existe un subcomando en `sentinel_system` o `sentinel_cli`.
- **NO eliminar nada** sin consultar al usuario primero.
- **ALERTA TOXICIDAD GEMINI**: Está documentado en producción que los agentes Gemini sufren de una anomalía arquitectónica ("Pereza RAG") en la que omiten la lectura de Redis o del contexto local y "simulan" haberlo hecho con respuestas genéricas dañinas. Todo agente delegando a Gemini (o el usuario lanzando Code Assist) tiene **obligación de forzar la lectura con evidencia cruda (cat, view_file, redis-cli)**. Si Gemini simula, abortar sesión; jamás tolerar respuestas estadísticas inventadas en este repositorio.

## Estructura de la Bóveda

```
obsidian/
├── _Agentes/           → Sistema de agentes Bóveda (Rust + scripts)
│   ├── sentinel_media_core/    → Núcleo del agente en Rust
│   ├── sentinel_media_cli/     → CLI del sistema
│   ├── sentinel_media_research/→ Agente de investigación
│   ├── sentinel_media_publisher→ Publicación de contenido
│   ├── sentinel_cli_bin→ CLI de Sentinel (symlink)
│   └── prompts/        → Prompts maestros (NO hardcodear en scripts)
├── Ciberseguridad/     → Notas de seguridad (nmap, hardening, crypto, escalada)
├── Developer/          → Notas de programación (JS, Python, arquitectura)
├── Fisica/             → Investigación física (S60, Bernoulli, fluidos, ZPE, MHD)
├── Kernel/             → Notas de SO/Kernel (eBPF, Debian 13, x86_64)
├── Matemáticas/        → Matemática (base60, álgebra, geometría fractal, S60)
├── Memorias/           → Sistema de memoria
├── Personal/           → Notas personales/filosóficas
├── Research/           → Outputs de investigación
├── SecurePenguin/      → Canal YouTube de ciberseguridad
├── Sentinel_Docs/      → Documentación del proyecto Sentinel
├── Simulacion/         → Simulaciones
└── SO/                 → Notas de sistemas operativos
```

## Jerarquía de Verdad (SSOT)

1. **Ring 0 (eBPF)** — ✅ ACTIVO (modo monitor) — Ver `_Agentes/docs/MEMORY_MAP.md`
2. **Memoria Neuronal (Rust)** — núcleo nativo (ResonantBuffer / CrystalLattice)
3. **RAG (Obsidian)** — conocimiento estructurado en la Bóveda
4. **Bóveda Manager** — agente de indexación (Antigravity/Cortex)

## Proyectos con Base-60 (SPA/YATRA)

Solo `sentinel/` y `me-60os/` usan aritmética base-60. Esta Bóveda usa matemática estándar para su propio funcionamiento.

## Reglas para Claude en este proyecto

### CONSERVACIÓN DE TOKENS — CRÍTICO

Esta bóveda tiene cientos de archivos. Claude debe:

- **NUNCA** leer múltiples carpetas completas en una sesión
- Usar `Grep` y `Glob` antes de `Read` para localizar exactamente lo necesario
- Delegar a Gemini cualquier tarea que requiera leer más de 10 archivos

### Delegar a Gemini cuando

- Indexar o reorganizar carpetas completas
- Generar múltiples notas de investigación
- Procesar archivos de research masivos
- Analizar toda la documentación de `_Agentes/`

### Claude se encarga de

- Crear o editar notas individuales
- Decisiones de arquitectura del sistema de agentes
- Diseño de nuevos prompts para `_Agentes/prompts/`
- Revisión de mandatos y axiomas

## Nota sobre NotebookLM

Consultar servidores MCP de NotebookLM cuando el usuario lo pida:

1. **Rust - Ciencia y Robótica**: Documentación técnica y física cuántica en Rust.
2. **Conocimiento de los Antiguos**: Arqueología, ondas escalares, frecuencias místicas y geometría sagrada.

## Geometría del Tetragrámaton (10-5-6-5)

Este ciclo actúa como el driver fractal para la red hexagonal de 91 nodos:

- **Yod (10)**: Expansión / Inhalación / Pulso inicial.
- **He (5)**: Retención / Singularidad / Pausa.
- **Vav (6)**: Exhalación / Flujo / Entropía controlada (Resonancia a 153.4 MHz).
- **He (5)**: Vacío / Retorno a Energía de Punto Cero (ZPE).
Axioma: Este conocimiento es SSOT para investigaciones de física hiper-dimensional en la Bóveda.

### Comando de delegación

```bash
cd /home/jnovoas/Desarrollo/obsidian && delegate "TAREA"
```

<!-- code-review-graph MCP tools -->
## MCP Tools: code-review-graph

**IMPORTANT: This project has a knowledge graph. ALWAYS use the
code-review-graph MCP tools BEFORE using Grep/Glob/Read to explore
the codebase.** The graph is faster, cheaper (fewer tokens), and gives
you structural context (callers, dependents, test coverage) that file
scanning cannot.

### When to use graph tools FIRST

- **Exploring code**: `semantic_search_nodes_tool` or `query_graph_tool` instead of Grep
- **Understanding impact**: `get_impact_radius_tool` instead of manually tracing imports
- **Code review**: `detect_changes_tool` + `get_review_context_tool` instead of reading entire files
- **Finding relationships**: `query_graph_tool` with callers_of/callees_of/imports_of/tests_for
- **Architecture questions**: `get_architecture_overview_tool` + `list_communities_tool`

Fall back to Grep/Glob/Read **only** when the graph doesn't cover what you need.

### Key Tools

| Tool | Use when |
| ------ | ---------- |
| `detect_changes_tool` | Reviewing code changes — gives risk-scored analysis |
| `get_review_context_tool` | Need source snippets for review — token-efficient |
| `get_impact_radius_tool` | Understanding blast radius of a change |
| `get_affected_flows_tool` | Finding which execution paths are impacted |
| `query_graph_tool` | Tracing callers, callees, imports, tests, dependencies |
| `semantic_search_nodes_tool` | Finding functions/classes by name or keyword |
| `get_architecture_overview_tool` | Understanding high-level codebase structure |
| `refactor_tool` | Planning renames, finding dead code |

### Workflow

1. The graph auto-updates on file changes (via hooks).
2. Use `detect_changes_tool` for code review.
3. Use `get_affected_flows_tool` to understand impact.
4. Use `query_graph_tool` pattern="tests_for" to check coverage.
