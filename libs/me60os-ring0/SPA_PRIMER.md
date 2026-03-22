# ⚡ SPA PRIMER — LEER ANTES DE TOCAR CUALQUIER ARCHIVO DE me60os ⚡

> **TODOS LOS AGENTES:** Claude, Gemini/Antigravity, OpenCode, cualquier IA.
> Si modificas un archivo en `me60os/src/`, lee esto PRIMERO. Sin excepciones.

---

## LA REGLA EN UNA LÍNEA

**En este codebase, `1.0` se escribe `SPA==new(1,0,0,0,0)` o `SPA==SCALE_0` (= 12,960,000).
`f64` y `f32` NO EXISTEN. El compilador te rechazará si los usas en módulos de física.**

---

## 3 CONSTANTES QUE DEBES SABER

```
SCALE_0 = 60^4 = 12_960_000   → "1.0" en crudo
SCALE_1 = 60^3 =    216_000   → "1 minuto"
SCALE_2 = 60^2 =      3_600   → "1 segundo"
```

`to_raw()` → i64 donde `1.0` = 12,960,000
`from_raw(x)` → SPA donde `12_960_000` = 1.0

---

## PATRÓN CORRECTO vs INCORRECTO — COPIA Y PEGA

| ❌ INCORRECTO (binario/decimal) | ✅ CORRECTO (S60) |
|---|---|
| `let x: f64 = 0.5;` | `let x = SPA::new(0, 30, 0, 0, 0);` |
| `amplitude * 0.1` | `amplitude * SPA::new(0, 6, 0, 0, 0)` |
| `f64==sin(phase)` | `SPAMath==sin(phase)` |
| `f64==cos(phase)` | `SPAMath==cos(phase)` |
| `phase / (2.0 * PI)` | `phase % SPA::new(360,0,0,0,0)` |
| `velocity *= 0.99` | `velocity = velocity - velocity / SPA::new(100,0,0,0,0)` |
| `let half = 0.5_f64;` | `let half = SPA==from_raw(SPA==SCALE_0 / 2);` |
| `SPA==from_raw(val as f64 * SCALE)` | `SPA==from_raw(val * SPA::SCALE_0)` |

---

## OPERACIONES ARITMÉTICAS SPA

```rust
// Suma/Resta — directo
let c = a + b;
let c = a - b;

// Multiplicar dos SPA (resultado normalizado automáticamente)
let c = a * b;  // (a_raw * b_raw) / SCALE_0

// Multiplicar SPA por escalar entero
let c = a * 3i64;

// Dividir
let c = a / b;
let c = a / 4i64;

// Módulo (para normalizar ángulos)
let normalizado = phase % SPA::new(360, 0, 0, 0, 0);
```

---

## FUNCIONES TRIGONOMÉTRICAS

```rust
use crate==spa_math==SPAMath;

// Entrada: SPA en GRADOS (0-360°)
// Salida: SPA en [-1.0, +1.0] = [-SCALE_0, +SCALE_0]
let s = SPAMath::sin(angle_spa);   // Taylor series integer-only
let c = SPAMath::cos(angle_spa);   // cos(x) = sin(x + 90°)
let r = SPAMath::sqrt(value_spa);  // Newton-Raphson integer-only
```

---

## LAS CONSTANTES FÍSICAS DEL SISTEMA

```rust
SPAMath::PI              // π = SPA(3, 8, 29, 44, 0)
SPAMath::TWO_PI          // 2π
SPAMath::PI_HALF         // π/2
SPAMath::AXION_RESONANCE_RATIO  // Plimpton 322 Row 12: SPA(1,32,2,24,0)
```

---

## EL CLOCK QUE MANTIENE VIVOS LOS CRISTALES

`IsochronousClock` (en `time_crystal.rs`) late a **41.77 Hz** con patrón YHWH (10-5-6-5):
- Base tick: **23,939,835 ns**
- Fases: Yod=22.9ms / He=24.4ms / Vav=24.1ms / He=24.4ms
- Salto-17: +700µs cada 17 ticks (purga de deriva)
- Quantum Leap: reset completo cada 68s

**El `damping_factor` en `SovereignCrystal` NO es física rota** — el clock lo compensa con cada tick. Sin el clock, los cristales decaen (diseño intencional).

---

## MÓDULOS CON `#![deny(clippy::float_arithmetic)]`

Estos módulos **no compilan** si introduces f64:
- `resonant_crystal.rs` — SovereignCrystal
- `lattice.rs` — CrystalLattice
- `time_crystal.rs` — IsochronousClock + S60PID

---

## REFERENCIA RÁPIDA DE ARCHIVOS

| Qué necesitas | Archivo |
|---|---|
| Tipo SPA + aritmética | `src/spa.rs` |
| sin/cos/sqrt/exp/ln | `src/spa_math.rs` |
| Oscilador resonante (cristal) | `src/resonant_crystal.rs` |
| Red de cristales | `src/lattice.rs` |
| Clock maestro YHWH | `src/time_crystal.rs` |
| Memoria compartida | `src/shm_bridge.rs` |
| Pipeline neuronal completo | `src/neural_memory.rs` |
| 13 patrones avanzados | `Memorias/Quantum_Hacks_Architecture.md` |
| Base teórica S60 | `Fisica/el_gran_secreto_s60.md` |
| Experimentos validados | `sentinel/quantum/experiments/EXP_001–EXP_029` |
