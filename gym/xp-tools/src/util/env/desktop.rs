use std::env::var;

pub fn get_env_variable(var_name: &str) -> Option<String> {
  match var(var_name) {
    Ok(var) => Some(var),
    Err(_) => None
  }
}