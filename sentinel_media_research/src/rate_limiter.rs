use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// Google AI Free Tier Limits:
// 15 RPM (Requests Per Minute)
// 1M TPM (Tokens Per Minute) - TPM is harder to track without response usage metadata, focusing on RPM first.
const DEFAULT_RPM_LIMIT: u64 = 14; // conservative buffer (15 is max)

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KeyState {
    pub last_window_start: u64, // Unix timestamp in seconds
    pub request_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RateLimiterData {
    keys: HashMap<String, KeyState>,
}

pub struct RateLimiter {
    data: RateLimiterData,
    path: PathBuf,
}

impl RateLimiter {
    pub fn new() -> Self {
        let path = PathBuf::from("/home/jnovoas/Obsidian/_Agentes/sentinel_rate_limits.json");
        let data = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            RateLimiterData::default()
        };

        Self { data, path }
    }

    fn get_current_minute_window(&self) -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let secs = since_the_epoch.as_secs();
        // Return the start of the current minute
        secs - (secs % 60)
    }

    fn calculate_hash<T: Hash>(&self, t: &T) -> String {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        format!("{:x}", s.finish())
    }

    // Returns true if the key is allowed to be used
    pub fn list_available_keys(&mut self, all_keys: &[String]) -> Vec<String> {
        let current_window = self.get_current_minute_window();
        let mut available = Vec::new();

        for key in all_keys {
            let key_id = self.calculate_hash(key);

            let state = self.data.keys.entry(key_id.clone()).or_insert(KeyState {
                last_window_start: current_window,
                request_count: 0,
            });

            // Reset if new window
            if state.last_window_start != current_window {
                state.last_window_start = current_window;
                state.request_count = 0;
            }

            if state.request_count < DEFAULT_RPM_LIMIT {
                available.push(key.clone());
            } else {
                // Key is hot
                continue;
            }
        }

        // Save state (resetting counters)
        self.save();

        available
    }

    pub fn record_usage(&mut self, key: &str) {
        let current_window = self.get_current_minute_window();
        let key_id = self.calculate_hash(&key.to_string());

        let state = self.data.keys.entry(key_id).or_insert(KeyState {
            last_window_start: current_window,
            request_count: 0,
        });

        if state.last_window_start != current_window {
            state.last_window_start = current_window;
            state.request_count = 0;
        }

        state.request_count += 1;
        self.save();
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.path, json);
        }
    }
}
