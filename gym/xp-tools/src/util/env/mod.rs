#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;

pub fn get_env_num(var_name: &str) -> Option<f64> {
  match get_env_variable(var_name) {
    Some(var_val) => Some(var_val.parse::<f64>().unwrap()),
    None => None
  }
}