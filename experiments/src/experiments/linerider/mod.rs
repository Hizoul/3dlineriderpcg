#[cfg(not(target_arch = "wasm32"))]
pub mod post_analysis;
#[cfg(not(target_arch = "wasm32"))]
pub mod heuristic_log;
#[cfg(not(target_arch = "wasm32"))]
pub mod booster_strength_exp;