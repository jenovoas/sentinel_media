# Calendario YouTube Factory

> **Estado:** En desarrollo. Ningún canal activo aún.
> Activar cuando el sistema de generación de contenido (`sentinel_media_media` + `sentinel_media_publisher`) esté operativo.

---

## Canales Planificados

| Canal | Nicho | Carpeta Bóveda | Estado |
| :--- | :--- | :--- | :--- |
| 🐧 **SecurePenguin** | Ciberseguridad / Linux | `SecurePenguin/` | ⏳ En desarrollo |
| 🌀 **ZeroRing** | Física Avanzada | `Fisica/` | ⏳ En desarrollo |
| 🦀 **SentinelLabs** | Dev / OS Internals / Rust | `Developer/` | ⏳ En desarrollo |

---

## Distribución Semanal (Objetivo futuro)

> Activar cuando haya pipeline de producción funcional.

| Día | Canal | Tipo de Contenido | Horario (GMT-3) |
| :--- | :--- | :--- | :--- |
| Lunes | SentinelLabs | Short (Optimización/Rust) | 12:00 |
| Martes | SecurePenguin | Short (Cyber-Insight) | 12:00 |
| Miércoles | SentinelLabs | Video principal | 19:00 |
| Jueves | SecurePenguin | Video principal | 19:00 |
| Viernes | SentinelLabs | Short avanzado | 12:00 |
| Sábado | ZeroRing | Video principal (Física) | 19:00 |
| Domingo | SecurePenguin | Video principal | 12:00 |

---

## Pipeline de Producción (Diseño objetivo)

```
sentinel_media_scanner   → detecta notas listas
sentinel_media_research  → investiga y sintetiza guion
sentinel_media_media     → genera assets (video, imagen, audio)
sentinel_media_publisher → sube a YouTube con metadatos
```

**Estado del pipeline:** Módulos Rust compilando. Integración end-to-end pendiente.

---

## Prerrequisitos para activar canales

- [ ] `sentinel_media_media` — generación de video funcional
- [ ] `sentinel_media_publisher` — autenticación YouTube OAuth configurada (`channels.yaml`)
- [ ] Prompts de guion por canal en `_Agentes/prompts/`
- [ ] Al menos 4 videos en backlog antes del primer lanzamiento

---

## Contenido en preparación

### SecurePenguin
- Guion maestro XZ Backdoor — `SecurePenguin/Guiones/`
- Estrategia de nicho — `SecurePenguin/Estrategia_Nicho_SecurePenguin.md`

### ZeroRing
- [ ] Definir estética visual (abstracto/cosmológico)
- [ ] Seleccionar temas seminales (Entropía, Tiempo, Quantum, Base-60)

### SentinelLabs
- [ ] Roadmap de contenido Rust avanzado
- [ ] Guiones de OS Internals (Kernel, eBPF, Drivers)
