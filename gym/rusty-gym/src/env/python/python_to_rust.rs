use crate::{GymEnvironment, Space, Action, Observation, Step};
use pyo3::{prelude::*, types::{IntoPyDict, PyTuple, PyFloat}};
use ndarray::ArrayBase;
use xp_tools::rng::from_seed;
use rand::RngCore;
use rand_pcg::Pcg64Mcg;

#[derive(Debug)]
pub enum ActionConversion {
  SingleVecFloat,
  SingleFloat,
  SingleInteger
}

#[derive(Debug)]
pub struct PythonToRustGym {
  last_used_seed: u64,
  has_reset_since_seed_change: bool,
  pub python_env: PyObject,
  rng: Pcg64Mcg,
  action_conversion: ActionConversion
}

impl PythonToRustGym {
  /**
   * Calls `gym.make(env_name)` in Python to create the Environment
   **/
  pub fn from_str(env_name: &str, seed: Option<u64>) -> Result<PythonToRustGym, PyErr> {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
      let gym_env_module = py.import("gym.envs")?;
      let env_make_function = gym_env_module.getattr("make")?;
      let eval_dict = [("make", env_make_function)].into_py_dict(py);
      let python_env_any = py.eval(&format!("make('{}')", env_name), Some(eval_dict), None)?;
      let (rng, _) = from_seed(seed);
      let action_conversion = match env_name {
        "Pendulum-v0" => {ActionConversion::SingleVecFloat},
        _ => {ActionConversion::SingleInteger}
      };
      Ok(PythonToRustGym {
        last_used_seed: 0,
        has_reset_since_seed_change: false,
        python_env: python_env_any.to_object(py),
        rng,
        action_conversion
      })
    })
  }
}

pub fn python_space_to_rust_space(_py: Python, space: &PyAny) -> PyResult<Space> {
  let space_type = space.get_type().to_string();
  Ok(match space_type.as_ref() {
    "<class 'gym.spaces.discrete.Discrete'>" => {
      let size: i64 = space.getattr("n")?.extract()?;
      Space::Discrete(size)
    },
    "<class 'gym.spaces.box.Box'>" => {
      let shape: Vec<i64> = space.getattr("shape")?.extract()?;
      let low: Vec<f64> = space.getattr("low")?.getattr("tolist")?.call((), None)?.extract()?;
      let high: Vec<f64> = space.getattr("high")?.getattr("tolist")?.call((), None)?.extract()?;
      Space::BoxedWithRange(shape, low, high)
    }
    _ => {
      panic!("Unable to convert python Space {} to Rust Space", space_type);
    }
  })
}
impl GymEnvironment for PythonToRustGym {
  fn use_seed(&mut self, seed: u64) {
    let (new_rng, _) = from_seed(Some(seed));
    self.rng = new_rng;
    self.last_used_seed = seed;
    self.has_reset_since_seed_change = false;
    pyo3::prepare_freethreaded_python();
    let res: Result<(), PyErr> = Python::with_gil(|py| {
      let seed_dict = [("py_env", &self.python_env)].into_py_dict(py);
      py.eval(&format!("py_env.seed({})", seed), Some(seed_dict), None)?;
      Ok(())
    });
    res.expect("Can set seed in loaded python environment");
  }

  fn reset(&mut self) -> Observation {
    if self.has_reset_since_seed_change {
      let new_num = self.rng.next_u64();
      self.use_seed(new_num);
    } else {
      self.has_reset_since_seed_change = true;
    }
    // TODO: extracted value should be according to 
    pyo3::prepare_freethreaded_python();
    let res: Result<Vec<f64>, PyErr> = Python::with_gil(|py| {
      let seed_dict = [("py_env", &self.python_env)].into_py_dict(py);
      let res = py.eval("py_env.reset().tolist()", Some(seed_dict), None)?;
      res.extract()
    });
    let converted: Vec<f64> = res.expect("Can call reset in loaded python environment");
    ArrayBase::from(converted).into_dyn()
  }

  fn step(&mut self, action: &Action) -> Step {
    pyo3::prepare_freethreaded_python();
    let res: Result<Step, PyErr> = Python::with_gil(|py| {
      let action_value = match self.action_conversion {
        ActionConversion::SingleFloat => {
          let a = action[0];
          let py_act_val = PyFloat::new(py, a);
          py_act_val.as_ref()
        },
        ActionConversion::SingleVecFloat => {
          let a = vec![action[0]];
          let py_act_val: PyObject = a.into_py(py);
          py_act_val.into_ref(py)
        },
        ActionConversion::SingleInteger => {
          let a = action[0].round() as i64;
          let py_act_val: PyObject = a.into_py(py);
          py_act_val.into_ref(py)
        }
      };
      let seed_dict = [("py_env", self.python_env.as_ref(py)), ("action_value", action_value)].into_py_dict(py);
      let py_step_res = py.eval("py_env.step(action_value)", Some(seed_dict), None)?;

      let py_step_tuple: &PyTuple = py_step_res.downcast()?;
      // todo: flexible observation decoding based on shape
      let obs_dict = [("step_res", py_step_tuple)].into_py_dict(py);
      let py_obs = py.eval("step_res[0].tolist()", Some(obs_dict), None)?;
      let obs: Vec<f64> = py_obs.extract()?;
      let reward: f64 = py_step_tuple.get_item(1)?.extract()?;
      let is_done: bool = py_step_tuple.get_item(2)?.extract()?;

      Ok(Step {
        obs: ArrayBase::from(obs).into_dyn(),
        reward,
        is_done,
        action: action.clone()
      })
    });
    res.expect("Can run env.step() and extract the python values to Rust")
  }

  fn action_space(&self) -> Space {
    pyo3::prepare_freethreaded_python();
    let res: Result<Space, PyErr> = Python::with_gil(|py| {
      let any = self.python_env.as_ref(py);
      let py_action_space = any.getattr("action_space")?;
      let rust_space = python_space_to_rust_space(py, py_action_space)?;
      Ok(rust_space)
    });
    res.expect("Can get action space")
  }

  fn observation_space(&self) -> Space {
    pyo3::prepare_freethreaded_python();
    let res: Result<Space, PyErr> = Python::with_gil(|py| {
      let any = self.python_env.as_ref(py);
      let py_action_space = any.getattr("observation_space")?;
      let rust_space = python_space_to_rust_space(py, py_action_space)?;
      Ok(rust_space)
    });
    res.expect("Can get observation space")
  }
}

#[cfg(feature = "eval")]
use crate::ReplayableGymEnvironment;
#[cfg(feature = "eval")]
use std::collections::HashMap;
#[cfg(feature = "eval")]
impl ReplayableGymEnvironment for PythonToRustGym {
  fn get_used_seed(&mut self) -> u64 {self.last_used_seed}
  fn get_config(&mut self) -> HashMap<String, String> {HashMap::new()}
  fn load_config(&mut self, _config: &HashMap<String, String>) {}
  fn get_name(&self) -> String {"todo".to_owned()}
  fn finalize(&mut self, _algo_name: &str, _eval_run_id: &str) {}
}


#[cfg(test)]
pub mod test {
  use crate::GymEnvironment;
  use super::PythonToRustGym;
  use pyo3::prelude::*;
  use ndarray::ArrayBase;
  use insta::assert_debug_snapshot;
  #[test]
  fn python_to_rust_test() -> Result<(), PyErr> {
      // let mut python_env = PythonToRustGym::from_str("CartPole-v1", Some(424242))?;
      // assert_debug_snapshot!("cartpole_actionspace", python_env.action_space());
      // assert_debug_snapshot!("cartpole_observationspace", python_env.observation_space());
      // python_env.use_seed(424242);
      // assert_debug_snapshot!("cartpole_seededreset", python_env.reset());
      // let res = python_env.step(&ArrayBase::from(vec![1.0]).into_dyn());
      // assert_debug_snapshot!("stepres", res);
      Ok(())
  }
}

