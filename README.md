# Sentinel Media: El Enjambre Autónomo de Contenido

**Automatización de YouTube de Alta Fidelidad Impulsada por Rust y Vertex AI**

Sentinel Media no es solo un generador de videos; es un **Enjambre de Agentes Soberanos** diseñado para transformar el capital intelectual (notas de Obsidian) en contenido visual de alto impacto. Construido con Rust para un rendimiento extremo y con la IA de Vertex de Google para una generación de vanguardia, automatiza todo el ciclo de vida de un canal de YouTube.

## 🐝 El Enjambre Sentinel

Nuestra arquitectura sigue el patrón de **Inteligencia de Enjambre**, donde agentes especializados colaboran a través de una capa de memoria Ring 0 descentralizada:

- **Agente Escáner**: Analiza tu bóveda de Obsidian, identificando notas con alto "Potencial de Contenido" basadas en entropía y puntuaciones de preparación.
- **Agente de Investigación**: Impulsado por Gemini 2.0 Flash, realiza investigación profunda en la web y sintetiza guiones complejos a partir de tus notas originales.
- **Agente de Medios (Fábrica)**: Interactúa con **Vertex AI Veo 3.0** (Video) e **Imagen 3** (Imágenes) para generar activos cinematográficos a escala.
- **Agente Publicador**: Gestiona la autenticación OAuth2 y las subidas reanudables a múltiples canales de YouTube (SecurePenguin, ZeroRing, SentinelLabs).
- **Agente Verificador**: Garantiza la consistencia de la marca y la calidad técnica antes de cualquier publicación.

## 🚀 El Flujo de Trabajo: De la Nota al Viral

1. **Escribir**: Crea una nota en Obsidian (ej: `# Hardening Linux 2026`).
2. **Etiquetar**: Establece `status: UNISON` o `ready: true`.
3. **Ejecutar**: El enjambre detecta la nota, investiga el tema, genera un guion, renderiza el video a través de Remotion/Veo y lo publica.

## 🛠️ Núcleo Técnico (Infraestructura Fenix)

- **Memoria Ring 0**: Sincronización en sub-milisegundos mediante redes de cristal de `me60os`.
- **Orquestación en Rust**: Workspace multi-crate que garantiza "Cero Errores en Tiempo de Ejecución" y máxima concurrencia.
- **Monitoreo en Tiempo Real**: Una interfaz Tauri premium para rastrear el metabolismo de los agentes y la salud del pipeline.
- **Nativo de la Nube**: Integración perfecta con Google Cloud Platform (Vertex AI, GCS, YouTube Data API).

## 📂 Estructura del Repositorio

- `sentinel-media-gui/`: Panel de Control y Monitoreo (Tauri + React).
- `sentinel_media_core/`: Tipos unificados y lógica del enjambre.
- `sentinel_media_research/`: Generación de guiones impulsada por Gemini.
- `sentinel_media_media/`: Fábrica de video con Vertex AI / Veo 3.0.
- `sentinel_media_publisher/`: Motor de automatización de YouTube.
- `libs/me60os-ring0/`: Capa de memoria Ring 0.

## 📄 Licencia

Licenciado bajo la **Licencia Apache 2.0**. Sentinel Media está construido para que la comunidad de código abierto democratice la IA de contenido de grado profesional.

---
*Impulsado por el Enjambre Fenix - Sentinel Media v1.5*
