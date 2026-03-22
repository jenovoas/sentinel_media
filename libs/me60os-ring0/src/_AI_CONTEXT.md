# me60os — Contexto Técnico para Agentes IA

> **LEER ESTO ANTES DE TOCAR CUALQUIER ARCHIVO**
> Versión: 2026-03-15 | Calibración: EXP-009, EXP-012, EXP-014, EXP-029 portados a Rust

---

## 1. Regla de Oro — AXIOMA I

**`f64` y `f32` NO EXISTEN en este codebase.**

Los módulos de física tienen `#![deny(clippy::float_arithmetic)]`. Si introduces un float, **el compilador lo rechazará**.

| Incorrecto | Correcto |
|-----------|---------|
| `let x = 1.0f64;` | `let x = SPA::new(1,0,0,0,0);` |
| `f64==sin(angle)` | `SPAMath==sin(angle)` |
| `amplitude * 0.5` | `amplitude * SPA::new(0,30,0,0,0)` |
| `phase as f64 / 360.0` | `phase.to_raw() / (360 * SPA::SCALE_0)` |

`SPA::SCALE_0 = 60^4 = 12_960_000` = "1.0" en este sistema.

---

## 2. Mapa de módulos — qué hace cada archivo

| Archivo | Struct(s) | EXP | Propósito |
|---------|-----------|-----|-----------|
| `spa.rs` | `SPA` | — | Fixed-point base-60. SCALE_0 = 12_960_000 |
| `spa_math.rs` | `SPAMath` | Hack#2 | sin/cos Taylor series, zero f64 |
| `spa_complex.rs` | `ComplexSPA` | — | Números complejos SPA |
| `time_crystal.rs` | `IsochronousClock`, `S60PID` | 001,027 | DTC pump YHWH 10-5-6-5, ~41.77 Hz |
| `resonant_crystal.rs` | `SovereignCrystal` | 003,007 | Célula resonante: amplitude+phase+damping |
| `lattice.rs` | `CrystalLattice` | 002,012 | Red 1D, difusión anisotrópica, sector snap |
| `lattice.rs` | `SparseCrystalLattice` | 014 | Red 1D sparse HashMap, 99.9% RAM |
| `hex_lattice.rs` | `HexLattice` | 009,011 | Red 2D hexagonal 91 nodos, byte storage |
| `scheduler.rs` | `PortalDetector` | 028,029 | φ(t) Bio+Crystal+Venus, detecta portales |
| `scheduler.rs` | `AdiabticTaskQueue` | 029 | Cola cuántica, 43.6% ahorro energético |
| `neural_memory.rs` | `NeuralMemory` | — | Pipeline: LIF → Hebbian → Crystal |
| `lif_neuron.rs` | `LIFNeuron` | — | Neurona LIF + SynapticMatrix |
| `crystal_store.rs` | `CrystalStore` | — | Persistencia mmap .crystal |
| `shm_bridge.rs` | `SharedBuffer` | — | Hot Memory L1 /dev/shm/pai60_lattice |

---

## 3. Física del sistema — conceptos críticos

### IsochronousClock ES el reactor

El `damping_factor` en `SovereignCrystal` es **entropía intencional** (disipación física real).
El `IsochronousClock` la compensa con cada tick — es el DTC pump.
**La física NO está rota.** No "arregles" el damping sin entender esto.

### Sector snapping (EXP-012)

256 sectores × 1.40625°/sector = 360°. Cada sector = 1 byte de información en canal de fase.
- `SECTOR_WIDTH_RAW = 18_225_000` (1.40625° en raw)
- `DIFFUSION_THRESHOLD_RAW = 9_112_500` (~0.703°)
- Δφ ≤ threshold → ruido → difunde
- Δφ > threshold → límite de datos → bloquea

### HexLattice — almacenamiento de bytes

```rust
let mut mem = HexLattice::new(5);          // 91 nodos = ~91 bytes
mem.write_bytes(b"hola mundo");            // escribe
mem.stabilize_phase(5);                     // ECC: corrige ruido automáticamente
let out = mem.read_bytes(11);              // lee con corrección integrada
```

### PortalDetector — cuándo ejecutar tareas

```rust
let phi = PortalDetector::resonance(t_ns); // φ ∈ [-SCALE_0, +SCALE_0]
if PortalDetector::is_open(phi) {          // φ > 0.75
    // Ejecutar tareas costosas AQUÍ, no en valles
}
```

---

## 4. Constantes de referencia

| Constante | Valor | Significado |
|-----------|-------|-------------|
| `SPA::SCALE_0` | 12_960_000 | "1.0" en base-60 |
| `SECTORS` | 256 | Sectores de fase (= 1 byte) |
| `SECTOR_WIDTH_RAW` | 18_225_000 | 1.40625° en raw |
| `DIFFUSION_THRESHOLD_RAW` | 9_112_500 | ~0.703° |
| Clock base | 23_939_835 ns | ~41.77 Hz |
| Salto-17 | +700_000 ns | cada 17 ticks |
| Quantum Leap | 68s | reset QHC |
| T_Bio | 17_000_000_000 ns | 17s ciclo biológico |
| T_Crystal | 4_250_000_000 ns | 4.25s |
| T_Venus | 16_180_000_000 ns | 16.18s (φ golden ratio) |
| Portal threshold | 9_720_000 | 0.75 × SCALE_0 |

---

## 5. Tests de referencia — verificación rápida

```bash
# Todos los tests del proyecto
cargo test

# Solo lattice (EXP-012 + EXP-014)
cargo test lattice

# Solo hex_lattice (EXP-009)
cargo test hex_lattice

# Solo scheduler (EXP-029)
cargo test scheduler
```

Estado al 2026-03-15: **57+ tests, 0 failures**.

---

## 6. Blindaje anti-alucinación — reglas de acero

- **NO** introducir `f64` / `f32` en archivos con `#![deny(clippy::float_arithmetic)]`
- **NO** "arreglar" el `damping_factor` sin leer la arquitectura del clock (es intencional)
- **NO** reemplazar `SPAMath==sin()` por `libm==sin()` o cualquier float
- **NO** usar `Vec::new()` para lattices con > 1000 índices (usar `SparseCrystalLattice`)
- **NO** cambiar `SECTORS = 256` — rompe el encoding de 1 byte por nodo
- Si el compilador pide `f32`/`f64`, la lógica es incorrecta — volver a componentes enteros SPA

---

## 7. Referencia de onboarding

- **`../SPA_PRIMER.md`** — tabla patrones CORRECTO/INCORRECTO (OBLIGATORIO)
- **`_Agentes/ANTIGRAVITY_CONTEXT.md`** — estado del sistema y proyectos
- **`Memorias/Quantum_Hacks_Architecture.md`** — 13 hacks físicos (SSOT)
- **`sentinel/quantum/experiments/`** — experimentos EXP_001 a EXP_029
