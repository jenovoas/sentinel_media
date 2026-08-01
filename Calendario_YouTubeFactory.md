# Calendario YouTube Factory

> **Estado:** En desarrollo. Ningún canal activo aún.
> Activar cuando el sistema de generación de contenido (`sentinel_media_media` + `sentinel_media_publisher`) esté operativo.

---

## Canales Planificados

| Canal | Nicho | Carpeta Bóveda | Estado |
| :--- | :--- | :--- | :--- |
| 🐧 **SecurePenguin** | Ciberseguridad / Linux | `SecurePenguin/` | ⏳ En desarrollo |
| 💻 **CodePenguin** | Programación Ofensiva / JS | `CodePenguin/` | ⏳ En desarrollo |
| ⚛️ **QuantumPenguin** | Física Cuántica / S60 | `QuantumPenguin/` | ⏳ En desarrollo |
| 🔷 **SumerPenguin** | Matemática Base-60 / S60 | `SumerPenguin/` | ⏳ En desarrollo |
| 🐧 **KernelPenguin** | Linux / Kernel Internals | `KernelPenguin/` | ⏳ En desarrollo |
| ⚙️ **AutoPenguin** | Automatización / Pipelines | `AutoPenguin/` | ⏳ En desarrollo |

---

## Distribución Semanal (Objetivo futuro)

> Activar cuando haya pipeline de producción funcional.

| Día | Canal | Tipo de Contenido | Horario (GMT-3) |
| :--- | :--- | :--- | :--- |
| Lunes | KernelPenguin | Short (Internals/Rust) | 12:00 |
| Martes | SecurePenguin | Short (Cyber-Insight) | 12:00 |
| Miércoles | AutoPenguin | Video principal (Pipeline) | 19:00 |
| Jueves | SecurePenguin | Video principal | 19:00 |
| Viernes | CodePenguin | Short (Offensive JS) | 12:00 |
| Sábado | QuantumPenguin | Video principal (Física) | 19:00 |
| Domingo | SumerPenguin | Video principal (Base-60) | 12:00 |

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

### CodePenguin
- [ ] Roadmap JS ofensivo — Prototype Pollution, DOM Clobbering, XSS

### QuantumPenguin
- [ ] Definir estética visual (abstracto/cosmológico)
- [ ] Seleccionar temas seminales (Fase, S60, Zhang & Wang)

### SumerPenguin
- [ ] Roadmap matemática base-60 y axiomas S60
- [ ] Demos visuales de geometría sexagesimal

### KernelPenguin
- [ ] Roadmap de contenido Linux internals
- [ ] Guiones de OS Internals (Kernel, eBPF, privesc)

### AutoPenguin
- [ ] Roadmap de pipelines reproducibles
- [ ] Guiones de automatización (factory → publish end-to-end)
