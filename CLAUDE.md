# CLAUDE.md — Bóveda Obsidian (Segundo Cerebro)

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
