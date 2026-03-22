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

## 🛠️ Tecnologías

- **Rust**: Núcleo y agentes (concurrencia y seguridad).
- **me60os**: Capa de memoria Ring 0 para sincronización sub-milisegundo.
- **GCP**: Vertex AI (Veo, Gemini) y Cloud Storage.

## 📄 Licencia

Apache License 2.0

---
*Fenix Swarm - Sentinel Media v1.5.1*
