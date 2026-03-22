use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlimptonRatio {
    pub row: u32,
    pub s60_repr: String,
    pub decimal_approx: f32,
    pub is_axion: bool,
}

pub fn get_exact_ratios() -> Vec<PlimptonRatio> {
    vec![
        PlimptonRatio { row: 1, s60_repr: "1; 59, 0, 15".to_string(), decimal_approx: 1.983, is_axion: false },
        PlimptonRatio { row: 2, s60_repr: "1; 56, 56, 58, 15".to_string(), decimal_approx: 1.949, is_axion: false },
        PlimptonRatio { row: 3, s60_repr: "1; 55, 7, 41, 16".to_string(), decimal_approx: 1.918, is_axion: false },
        PlimptonRatio { row: 4, s60_repr: "1; 53, 10, 29, 32".to_string(), decimal_approx: 1.886, is_axion: false },
        PlimptonRatio { row: 5, s60_repr: "1; 48, 54, 1, 40".to_string(), decimal_approx: 1.815, is_axion: false },
        PlimptonRatio { row: 6, s60_repr: "1; 47, 6, 41, 40".to_string(), decimal_approx: 1.785, is_axion: false },
        PlimptonRatio { row: 7, s60_repr: "1; 43, 11, 56, 28".to_string(), decimal_approx: 1.719, is_axion: false },
        PlimptonRatio { row: 8, s60_repr: "1; 41, 33, 45, 14".to_string(), decimal_approx: 1.692, is_axion: false },
        PlimptonRatio { row: 9, s60_repr: "1; 38, 33, 36, 36".to_string(), decimal_approx: 1.642, is_axion: false },
        PlimptonRatio { row: 10, s60_repr: "1; 35, 10, 2, 28".to_string(), decimal_approx: 1.586, is_axion: false },
        PlimptonRatio { row: 11, s60_repr: "1; 33, 45, 0, 0".to_string(), decimal_approx: 1.562, is_axion: false },
        PlimptonRatio { row: 12, s60_repr: "1; 32, 2, 24, 0".to_string(), decimal_approx: 1.534, is_axion: true },
        PlimptonRatio { row: 13, s60_repr: "1; 27, 0, 3, 45".to_string(), decimal_approx: 1.450, is_axion: false },
        PlimptonRatio { row: 14, s60_repr: "1; 25, 48, 51, 36".to_string(), decimal_approx: 1.430, is_axion: false },
        PlimptonRatio { row: 15, s60_repr: "1; 23, 13, 46, 40".to_string(), decimal_approx: 1.387, is_axion: false },
    ]
}
