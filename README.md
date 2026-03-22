# Sentinel Media (Fenix)

**Enjambre Autónomo de Contenido | Rust & Vertex AI**

Sentinel Media transforma notas de Obsidian en videos de YouTube mediante un enjambre de agentes especializados y memorias Ring 0.

## 🐝 El Enjambre

Estructura modular de agentes coordinados por la infraestructura Fenix:

- **Escáner**: Identifica notas listas en Obsidian.
- **Investigación**: Crea guiones con Gemini 2.0 Flash.
- **Fábrica**: Genera video cinematográfico con **Veo 3.0**.
- **Publicador**: Automatiza subidas a YouTube con OAuth2.
- **Panel**: Interfaz Tauri para monitoreo en tiempo real.

## 🚀 Flujo Rápido

1. Crea nota en Obsidian.
2. Marca como `ready: true`.
3. El enjambre procesa, renderiza y publica automáticamente.

```bash
# PASO 1 — Escanear bóveda y detectar candidatos listos
sentinel scan --vault ~/mi-vault --min-score 0.9

# PASO 2 — Generar guiones e iniciar la cadena de producción
sentinel factory --research --provider gemini
```

## 📂 Estructura del Repositorio

| Directorio | Descripción |
|---|---|
| `cli/` | Punto de entrada principal. Orquesta todos los agentes mediante subcomandos (`scan`, `factory`, `research`, `status`). |
| `core/` | Tipos unificados, traits compartidos y lógica fundamental del enjambre. |
| `scanner/` | Identifica notas en la bóveda de Obsidian marcadas como `ready: true` y calcula su puntuación de prioridad. |
| `research/` | Genera guiones y dossiers usando Gemini 2.0 Flash / Perplexity. Soporta PDFs, ingesta de memoria y modo `--deep`. |
| `media/` | Fábrica de video. Orquesta la generación cinematográfica con Vertex AI (Veo 3.0) y renderizado local (NVENC). |
| `publisher/` | Motor de automatización de YouTube: sube el video, aplica metadatos y gestiona OAuth2. |
| `verifier/` | Valida la integridad de los activos generados antes de publicar. |
| `memory/` | Capa de memoria Ring 0: embeddings RAG, neuronas LIF y persistencia `CrystalLattice` (mmap). |
| `system/` | Asistente SysAdmin: traduce lenguaje natural a comandos seguros de Debian. |
| `agent-obsidian/` | Agente de integración con Obsidian (lecto-escritura de notas y metadatos). |
| `sentinel-media-gui/` | Panel de control Tauri + React para monitoreo en tiempo real del enjambre. |
| `sentinel-media-remotion/` | Compositor de video alternativo basado en Remotion (Node.js). |

## 🛠️ Tecnologías

- **Rust**: Núcleo y agentes (concurrencia y seguridad).
- **me60os**: Capa de memoria Ring 0 para sincronización sub-milisegundo.
- **GCP**: Vertex AI (Veo 3.0, Gemini 2.0) y Cloud Storage.
- **Tauri + React**: Panel de control nativo multiplataforma.

## 📄 Licencia

Apache License 2.0 — Construido para que la comunidad open source democratice la IA de contenido de grado profesional.

---
*Fenix Swarm - Sentinel Media v1.5.1*
