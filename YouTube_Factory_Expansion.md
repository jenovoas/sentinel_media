# YouTube Factory — Arquitectura del Sistema

> **Estado:** Sistema en desarrollo. Ver `_Agentes/PLAN_SANEAMIENTO_2026-03-15.md` para el roadmap de implementación.

---

## Canales

| Canal | Nicho | Identidad | Carpeta |
| :--- | :--- | :--- | :--- |
| **SecurePenguin** | Ciberseguridad / Linux | El Ingeniero Soberano | `SecurePenguin/` |
| **CodePenguin** | Programación Ofensiva / JS | El Code Hacker | `CodePenguin/` |
| **QuantumPenguin** | Física Cuántica / S60 | El Físico Soberano | `QuantumPenguin/` |
| **SumerPenguin** | Matemática Base-60 / S60 | El Matemático Soberano | `SumerPenguin/` |
| **KernelPenguin** | Linux / Kernel Internals | El Root Curioso | `KernelPenguin/` |
| **AutoPenguin** | Automatización / Pipelines | El Automatizador Soberano | `AutoPenguin/` |

---

## Arquitectura del Pipeline

```
Bóveda (notas Obsidian)
    ↓ sentinel_media_scanner — detecta notas con score alto
    ↓ sentinel_media_research — genera guion con Gemini API
    ↓ sentinel_media_media — genera video/imagen/audio
    ↓ sentinel_media_publisher — sube a YouTube (Rust + Node.js)
```

**Módulos Rust:** Ver `_Agentes/agents.md`
**Prompts por agente:** Ver `_Agentes/prompts/`

---

## Estado de Implementación

| Componente | Estado |
|-----------|--------|
| Módulos Rust (compilación) | ✅ Todos limpios |
| Integración end-to-end | ❌ Pendiente |
| Autenticación YouTube OAuth | ❌ Pendiente |
| `channels.yaml` | ❌ Pendiente crear |
| Prompts de guion por canal | ⏳ Parcial (`_Agentes/prompts/youtube_architect.md`) |
| Backlog de contenido | ⏳ Parcial (SecurePenguin) |

---

## Próximos pasos

1. Completar integración `sentinel_media_scanner` → `sentinel_media_research`
2. Configurar OAuth YouTube en `sentinel_media_publisher`
3. Crear `channels.yaml` con los 3 canales
4. Probar pipeline completo con un video de SecurePenguin (tiene más contenido preparado)
5. Activar distribución semanal cuando haya 4 videos en backlog
