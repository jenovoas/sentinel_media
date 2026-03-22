//! # 🛡️ TRUTHSYNC: SEMANTIC & ENTROPIC FIREWALL 🛡️
//!
//! Implementation of the Sentinel "SCV" architecture.
//! Acts as a membrane involved in filtering information before it reaches the Cognitive Core.
//!
//! Two Layers:
//! 1. **Semantic Firewall**: Regex-based pattern matching (Truth vs Falsehood).
//! 2. **Entropic Firewall**: Shannon Entropy validation (Signal vs Noise).

use crate::bio::SoulVerifier; // Reuse Entropy Logic
use crate::spa::SPA;
use regex::RegexSet;

pub struct SemanticFirewall {
    // Whitelist patterns (Constructive/Truthful)
    #[allow(dead_code)]
    allowed_patterns: RegexSet,
    // Blacklist patterns (Destructive/False/Noise)
    blocked_patterns: RegexSet,
}

impl SemanticFirewall {
    pub fn new() -> Self {
        // Default rules (Example - to be expanded)
        let allowed = RegexSet::new(&[
            r"ME-60OS",
            r"Resonance",
            r"Truth",
            r"Physics",
            r"System Stable",
        ])
        .unwrap();

        let blocked =
            RegexSet::new(&[r"Error", r"Failure", r"Corruption", r"Panic", r"Attack"]).unwrap();

        Self {
            allowed_patterns: allowed,
            blocked_patterns: blocked,
        }
    }

    pub fn has_keywords(&self, text: &str) -> bool {
        self.allowed_patterns.is_match(text)
    }

    /// Checks if content passes the semantic filter
    pub fn verify(&self, text: &str) -> bool {
        // Block if matches blacklist
        if self.blocked_patterns.is_match(text) {
            return false;
        }
        true
    }
}

pub struct EntropicFirewall;

impl EntropicFirewall {
    /// Verifies if the information density is sufficient (neither random noise nor static repetition).
    /// Uses Shannon Entropy from Bio-Resonator.
    pub fn verify(signal: &[SPA]) -> bool {
        let metrics = SoulVerifier::analyze(signal);
        metrics.is_alive
    }

    /// Calculate raw Shannon Entropy
    pub fn calculate_entropy(text: &str) -> f64 {
        if text.is_empty() {
            return 0.0;
        }

        let mut counts = [0usize; 256];
        let mut total = 0;

        for b in text.bytes() {
            counts[b as usize] += 1;
            total += 1;
        }

        let mut entropy = 0.0;
        for &count in &counts {
            if count == 0 {
                continue;
            }
            let p = count as f64 / total as f64;
            entropy -= p * p.ln();
        }

        // Convert to base 2 bits approximation (using ln(2) = 0.693)
        entropy / std::f64::consts::LN_2
    }

    /// Calculate text entropy validation
    pub fn verify_text(text: &str) -> bool {
        let entropy = Self::calculate_entropy(text);
        // Valid range for human-readable technical text
        entropy > 2.0 && entropy < 6.0
    }
}

pub struct ScvEngine {
    semantic: SemanticFirewall,
    #[allow(dead_code)]
    entropic: EntropicFirewall,
}

impl ScvEngine {
    pub fn new() -> Self {
        Self {
            semantic: SemanticFirewall::new(),
            entropic: EntropicFirewall,
        }
    }

    pub fn verify(&self, text: &str) -> bool {
        self.analyze(text).0
    }

    /// Returns (is_valid, score, entropy, has_keywords)
    pub fn analyze(&self, text: &str) -> (bool, f64, f64, bool) {
        if text.trim().is_empty() {
            return (false, 0.0, 0.0, false);
        }

        let blocked = self.semantic.blocked_patterns.is_match(text);
        if blocked {
            return (false, 0.1, 0.0, false);
        }

        let entropy = EntropicFirewall::calculate_entropy(text);
        let has_keywords = self.semantic.has_keywords(text);

        let valid_entropy = if text.len() < 5 {
            true
        } else {
            entropy > 2.0 && entropy < 6.0
        };

        let mut score = 0.5; // Neutral baseline
        if has_keywords {
            score += 0.3;
        } // +0.3 for keywords
        if valid_entropy {
            score += 0.2;
        } // +0.2 for good entropy

        // Cap at 1.0
        if score > 1.0 {
            score = 1.0;
        }

        let is_valid = !blocked && valid_entropy;

        (is_valid, score, entropy, has_keywords)
    }
}
