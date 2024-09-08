use pyo3::prelude::*;
use std::collections::HashMap;
use crate::simulator::LineRiderSim;
use crate::env::LineRider3DEnv;
use rusty_gym::{ReplayableGymEnvironment, env::python::RustToPyGym};

#[pyfunction]
pub fn make_env(config_opt: Option<HashMap<String, String>>, record_with_id: Option<String>) -> RustToPyGym {
  let config = config_opt.unwrap_or(HashMap::new());
  
  let sim: LineRiderSim = LineRiderSim::new(false);
  let mut env = LineRider3DEnv::new(sim, None);
  env.load_config(&config);

  Python::with_gil(|py| {
    let pyenv = RustToPyGym::new(py, Box::new(env), record_with_id);
    pyenv
  })
}

#[pymodule]
fn linerider(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(make_env, m)?)?;
    Ok(())
}
