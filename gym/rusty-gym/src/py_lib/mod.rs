use pyo3::prelude::*;
use crate::{util::rng::RngType, env::{
  control::CartpoleEnv,
  python::rust_to_python::RustToPyGym,
  // python::reset::{ResettablePyGym, DummyPos}
}};

// #[pyclass(unsendable, extends=ResettablePyGym, module = "rusty_gym")]
// pub struct ResettableCartpole {
// }


// #[pymethods]
// impl ResettableCartpole {
//   #[new]
//   fn new(py: Python) -> (Self, ResettablePyGym) {
//     let cp_env = CartpoleEnv::new(None, None);
//     (ResettableCartpole {}, ResettablePyGym::new(py, Box::new(cp_env)))
//   }
// }

#[pymodule]
#[pyo3(name = "rusty_gym")]
pub fn rusty_gym(_py: Python, module: &PyModule) -> PyResult<()> {
  #[pyfn(module)]
  #[pyo3(name = "cartpole_env")]
  fn cartpole_env(py: Python, rng_to_use: u8) -> RustToPyGym {
    let rng_type = match rng_to_use {
      1 => {RngType::Pcg64Mcg},
      2 => {RngType::Pcg64},
      3 => {RngType::Pcg32},
      _ => {RngType::Mt19937}
    };
    let cp_env = CartpoleEnv::new(None, Some(rng_type));
    let none_string: Option<String> = None;
    RustToPyGym::new(py, Box::new(cp_env), none_string)
  }
  // #[pyfn(module)]
  // #[pyo3(name = "resettable_cartpole_env")]
  // fn resettable_cartpole_env(py: Python, rng_to_use: u8) -> ResettablePyGym {
  //   let rng_type = match rng_to_use {
  //     1 => {RngType::Pcg64Mcg},
  //     2 => {RngType::Pcg64},
  //     3 => {RngType::Pcg32},
  //     _ => {RngType::Mt19937}
  //   };
  //   let cp_env = CartpoleEnv::new(None, Some(rng_type));
  //   ResettablePyGym::new(py, Box::new(cp_env))
  // }
  // module.add_class::<DummyPos>()?;
  // module.add_class::<ResettableCartpole>()?;
  Ok(())
}