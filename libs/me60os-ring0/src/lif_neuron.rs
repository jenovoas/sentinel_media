//! # 🧠 Red Neuronal de Impulsos LIF (Leaky Integrate-and-Fire)
//!
//! Implementación SNN en aritmética Base-60 pura (SPA).
//! Sin decimales, sin floats. Toda la física neuronal en campo sexagesimal.
//!
//! ## Modelo LIF
//! ```text
//! V(t+1) = V(t) - V(t)*leak + I(t)
//! Dispara si V >= threshold → V = 0, refractory = 3 ticks
//! ```
//!
//! ## Aprendizaje Hebbiano
//! ```text
//! w[i][j] += eta   cuando pre[i] y post[j] disparan juntos
//! ```
//!
//! ## Plasticidad Homeostática
//! Cada 60 ticks: ajusta threshold según tasa de disparo real vs objetivo (1/60)

use crate::spa::SPA;

// =============================================================================
// NEURONA LIF
// =============================================================================

/// Neurona Leaky Integrate-and-Fire en campo SPA (Base-60).
#[derive(Clone)]
pub struct LIFNeuron {
    /// Potencial de membrana actual
    pub membrane: SPA,
    /// Umbral de disparo (se ajusta homeostáticamente)
    pub threshold: SPA,
    /// Factor de fuga por tick: 1/60 ≈ 0.0167
    pub leak: SPA,
    /// Ticks de período refractario restantes
    pub refractory: u8,
    /// Disparos en la ventana homeostática actual
    spike_window: u32,
    /// Ticks totales para sincronizar ventana homeostática
    tick_count: u32,
}

impl LIFNeuron {
    /// Umbral inicial: 1 unidad SPA
    const THRESHOLD_INIT: SPA = SPA::new(1, 0, 0, 0, 0);
    /// Fuga: 1/60 por tick
    const LEAK_FACTOR: SPA = SPA::new(0, 1, 0, 0, 0);
    /// Delta homeostático: 1/3600 (ajuste fino del umbral)
    const HOMEOSTATIC_DELTA: SPA = SPA::new(0, 0, 1, 0, 0);
    /// Período refractario: 3 ticks ≈ 72ms a 24ms/tick
    const REFRACTORY_TICKS: u8 = 3;
    /// Ventana homeostática: 60 ticks (un ciclo YHWH completo)
    const HOMEOSTATIC_WINDOW: u32 = 60;
    /// Tasa objetivo: 1 disparo por ventana
    const TARGET_SPIKES: u32 = 1;

    pub fn new() -> Self {
        Self {
            membrane: SPA::zero(),
            threshold: Self::THRESHOLD_INIT,
            leak: Self::LEAK_FACTOR,
            refractory: 0,
            spike_window: 0,
            tick_count: 0,
        }
    }

    /// Integra entrada y avanza un tick. Devuelve `true` si la neurona disparó.
    pub fn step(&mut self, input: SPA) -> bool {
        self.tick_count += 1;

        // Período refractario: la neurona no integra
        if self.refractory > 0 {
            self.refractory -= 1;
            self.homeostatic_update();
            return false;
        }

        // V(t+1) = V(t) - V(t)*leak + input
        let leak_amount = self.membrane * self.leak;
        self.membrane = self.membrane - leak_amount + input;

        // Disparo
        let fired = self.membrane >= self.threshold;
        if fired {
            self.membrane = SPA::zero();
            self.refractory = Self::REFRACTORY_TICKS;
            self.spike_window += 1;
        }

        self.homeostatic_update();
        fired
    }

    /// Ajusta el umbral al final de cada ventana de 60 ticks.
    /// Si dispara demasiado → sube umbral. Si dispara poco → baja umbral.
    fn homeostatic_update(&mut self) {
        if self.tick_count % Self::HOMEOSTATIC_WINDOW != 0 {
            return;
        }
        let delta = Self::HOMEOSTATIC_DELTA;
        if self.spike_window > Self::TARGET_SPIKES {
            self.threshold = self.threshold + delta;
        } else if self.spike_window < Self::TARGET_SPIKES && self.threshold > delta {
            self.threshold = self.threshold - delta;
        }
        self.spike_window = 0;
    }
}

impl Default for LIFNeuron {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// MATRIZ SINÁPTICA (HEBBIAN)
// =============================================================================

/// Pesos sinápticos con aprendizaje hebbiano en SPA.
/// "Neurons that fire together, wire together."
pub struct SynapticMatrix {
    /// Pesos w[pre][post] en SPA
    pub weights: Vec<Vec<SPA>>,
    /// Tasa de aprendizaje: 1/3600 por co-disparo
    pub eta: SPA,
}

impl SynapticMatrix {
    pub fn new(size: usize) -> Self {
        Self {
            weights: vec![vec![SPA::zero(); size]; size],
            eta: SPA::new(0, 0, 1, 0, 0), // eta = 1/3600
        }
    }

    /// Refuerza conexiones entre neuronas que co-dispararon.
    pub fn hebbian_update(&mut self, pre_spikes: &[bool], post_spikes: &[bool]) {
        let n = pre_spikes.len().min(post_spikes.len()).min(self.weights.len());
        for i in 0..n {
            if !pre_spikes[i] {
                continue;
            }
            for j in 0..n {
                if post_spikes[j] {
                    self.weights[i][j] = self.weights[i][j] + self.eta;
                }
            }
        }
    }

    /// Propaga señales a través de la matriz de pesos.
    pub fn propagate(&self, inputs: &[SPA]) -> Vec<SPA> {
        let n = inputs.len().min(self.weights.len());
        let mut outputs = vec![SPA::zero(); n];
        for j in 0..n {
            for i in 0..n {
                outputs[j] = outputs[j] + inputs[i] * self.weights[i][j];
            }
        }
        outputs
    }
}

// =============================================================================
// CAPA NEURONAL
// =============================================================================

/// Capa de neuronas LIF con sinapsis hebbianas.
pub struct NeuronLayer {
    pub neurons: Vec<LIFNeuron>,
    pub synapses: SynapticMatrix,
}

impl NeuronLayer {
    pub fn new(size: usize) -> Self {
        Self {
            neurons: (0..size).map(|_| LIFNeuron::new()).collect(),
            synapses: SynapticMatrix::new(size),
        }
    }

    /// Paso completo: integra entradas directas + sinápticas, dispara, aprende.
    pub fn step(&mut self, raw_inputs: &[SPA]) -> Vec<bool> {
        let n = self.neurons.len();

        // Señales sinápticas (aprendidas) sumadas a las entradas directas
        let syn = self.synapses.propagate(raw_inputs);
        let combined: Vec<SPA> = (0..n.min(raw_inputs.len()))
            .map(|i| raw_inputs[i] + syn[i])
            .collect();

        // Integrar y disparar
        let mut spikes = vec![false; n];
        for i in 0..n.min(combined.len()) {
            spikes[i] = self.neurons[i].step(combined[i]);
        }

        // Aprendizaje hebbiano post-disparo
        let pre: Vec<bool> = raw_inputs.iter().map(|s| s.to_raw() > 0).collect();
        self.synapses.hebbian_update(&pre, &spikes);

        spikes
    }

    /// Número de neuronas activas (no en refractario)
    pub fn active_count(&self) -> usize {
        self.neurons.iter().filter(|n| n.refractory == 0).count()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lif_dispara_con_input_suficiente() {
        let mut n = LIFNeuron::new();
        // Integrar umbral completo en un tick
        let fired = n.step(SPA::new(1, 0, 0, 0, 0));
        assert!(fired, "debe disparar cuando input >= threshold");
        assert_eq!(n.membrane, SPA::zero(), "membrana resetea tras disparo");
        assert_eq!(n.refractory, 3, "período refractario activo");
    }

    #[test]
    fn test_lif_no_dispara_bajo_umbral() {
        let mut n = LIFNeuron::new();
        let fired = n.step(SPA::new(0, 1, 0, 0, 0)); // input = 1/60 << threshold
        assert!(!fired);
    }

    #[test]
    fn test_lif_refractario_bloquea_disparo() {
        let mut n = LIFNeuron::new();
        n.step(SPA::new(1, 0, 0, 0, 0)); // primer disparo
        // Siguiente tick con input alto: refractario activo
        let fired = n.step(SPA::new(2, 0, 0, 0, 0));
        assert!(!fired, "refractario debe bloquear el disparo");
    }

    #[test]
    fn test_hebbiano_refuerza_peso() {
        let mut m = SynapticMatrix::new(2);
        let pre = [true, false];
        let post = [false, true];
        m.hebbian_update(&pre, &post);
        // w[0][1] debería haber aumentado (pre[0] y post[1] co-dispararon)
        assert!(m.weights[0][1].to_raw() > 0, "peso debe aumentar tras co-disparo");
        // w[0][0], w[1][0], w[1][1] deben ser cero
        assert_eq!(m.weights[0][0].to_raw(), 0);
    }

    #[test]
    fn test_capa_propaga_spikes() {
        let mut layer = NeuronLayer::new(4);
        let inputs = vec![
            SPA::new(1, 0, 0, 0, 0), // input alto → disparo
            SPA::zero(),
            SPA::zero(),
            SPA::zero(),
        ];
        let spikes = layer.step(&inputs);
        assert!(spikes[0], "neurona 0 debe disparar con input = 1.0");
    }
}
