use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyList, IntoPyDict, PyDict};
use crate::{Space, ReplayableGymEnvironment, GymRecorder};
use super::util::*;
use ndarray::ArrayBase;
use std::collections::HashMap;
use std::time::Instant;

/**
 * Functions
 * close
 * render
 * step
 * reset
 * seed
 * unwrapped
 * __str__
 * __enter__
 * __exit__
 * 
 **/
#[pyclass(unsendable)]
pub struct RustToPyGym {
  #[pyo3(get)]
  metadata: PyObject,
  #[pyo3(get)]
  action_space: PyObject,
  #[pyo3(get)]
  observation_space: PyObject,
  #[pyo3(get)]
  reward_range: PyObject,
  #[pyo3(get)]
  spec: PyObject,
  #[pyo3(get)]
  time_in_env: u128,
  rust_env: Box<dyn ReplayableGymEnvironment>,
  obs_shape: Option<Vec<i64>>
}


#[pymethods]
impl RustToPyGym {
  pub fn reset(&mut self, py: Python, seed: Option<i64>, additional_options: Option<HashMap<String, String>>) -> PyResult<PyObject> {
    let start = Instant::now();
    let reset_result = self.rust_env.reset();
    let observation = match &self.obs_shape {
      Some(shape) => {
        let numpy_module = py.import("numpy")?;
        let reshape = numpy_module.getattr("reshape")?;
        let to_convert_py = PyList::new(py, &reset_result);
        let shape_py = PyList::new(py, shape);
        reshape.call(PyTuple::new(py, &[to_convert_py, shape_py]), None)?
      },
      None => {PyList::new(py, reset_result.into_raw_vec()).as_ref()}
    };
    let res = observation.to_object(py);
    let info = PyDict::new(py).into();
    self.time_in_env += start.elapsed().as_nanos();
    let result_tuple = PyTuple::new(py, vec![res, info]);
    let obj: PyObject = result_tuple.to_object(py);
    PyResult::Ok(obj)
  }
  pub fn step(&mut self, py: Python, arg: &PyAny) -> PyResult<PyObject> {
    let start = Instant::now();
    let action = {
      match self.rust_env.action_space() {
        Space::Discrete(_) => {
          let actual_action_opt: PyResult<Vec<f64>> = arg.extract();
          if let Ok(actual_action) = actual_action_opt {
            ArrayBase::from(actual_action).into_dyn()
          } else {
            let actual_action: i64 = arg.extract()?;
            ArrayBase::from(vec![actual_action as f64]).into_dyn()
          }
        }
        _ => {
          let actual_action: Vec<f64> = arg.extract()?;
          ArrayBase::from(actual_action).into_dyn()
        }
      }
    };
    let step_res = self.rust_env.step(&action);
    let observation = match &self.obs_shape {
      Some(shape) => {
        let numpy_module = py.import("numpy")?;
        let reshape = numpy_module.getattr("reshape")?;
        let to_convert_py = PyList::new(py, &step_res.obs);
        let shape_py = PyList::new(py, shape);
        reshape.call(PyTuple::new(py, &[to_convert_py, shape_py]), None)?
      },
      None => {PyList::new(py, step_res.obs.into_raw_vec()).as_ref()}
    };
    let reward_tuple = PyTuple::new(py, &[step_res.reward]);
    let done_tuple = PyTuple::new(py, &[step_res.is_done]);
    let reward = reward_tuple.get_item(0).unwrap();
    let done = done_tuple.get_item(0).unwrap();
    let truncated_tuple = PyTuple::new(py, &[step_res.is_done]);
    let truncated = truncated_tuple.get_item(0).unwrap();
    let additional_info: &PyDict = [("empty", "info")].into_py_dict(py);
    let result = PyTuple::new(py, &[observation, reward, done, truncated, additional_info.as_ref()]);
    let res = PyResult::Ok(result.to_object(py));
    self.time_in_env += start.elapsed().as_nanos();
    res
  }
  pub fn get_config(&mut self) -> HashMap<String, String> {
    self.rust_env.get_config()
  }
  pub fn load_config(&mut self, config: HashMap<String, String>) {
    self.rust_env.load_config(&config);
  }
  pub fn get_used_seed(&mut self) -> u64 {
    self.rust_env.get_used_seed()
  }
  pub fn finalize(&mut self, algo_name: &str, eval_run_id: &str) {
    self.rust_env.finalize(algo_name, eval_run_id);
  }
}

impl RustToPyGym {
  pub fn new(py: Python, mut rust_env: Box<dyn ReplayableGymEnvironment>, recording_id: Option<String>) -> Self {
    if recording_id.is_some() {
      rust_env = Box::new(GymRecorder::new(rust_env, recording_id.clone()));
    }
    let start = Instant::now();
    let action_space = rust_space_to_gym_space(py, rust_env.action_space()).unwrap();
    let action_space = action_space.to_object(py);
    let observation_space = rust_space_to_gym_space(py, rust_env.observation_space()).unwrap();
    let observation_space = observation_space.to_object(py);
    let spec = observation_space.to_object(py);
    //TODO: make the reward range defineable
    let b = PyList::new(py, &[-2, 2]);
    let reward_range = b.to_object(py);
    let metadata_b = PyList::new(py, &[-2, 2]);
    let metadata = metadata_b.to_object(py);
    let obs_shape: Option<Vec<i64>> = get_obs_shape(rust_env.observation_space());
    let mut res = RustToPyGym {
      action_space, observation_space, obs_shape, reward_range,
      spec, rust_env, metadata, time_in_env: 0
    };
    res.time_in_env = start.elapsed().as_nanos();
    res
  }
}