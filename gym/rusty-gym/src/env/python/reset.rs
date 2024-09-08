use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyList, IntoPyDict, PyDict};
use crate::{Space, reset::ResettableGymEnvironment};
use ndarray::ArrayBase;
use std::time::Instant;
use std::collections::HashMap;

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
#[pyclass(unsendable, subclass)]
pub struct ResettablePyGym {
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
  #[pyo3(get,set)]
  rooms: PyObject,
  rust_env: Box<dyn ResettableGymEnvironment>
}

#[pyclass(module = "rusty_gym")]
pub struct DummyPos {
  #[pyo3(get, set)]
  level: i64,
  #[pyo3(get, set)]
  score: i64,
  #[pyo3(get, set)]
  room: i64,
  #[pyo3(get, set)]
  x: i64,
  #[pyo3(get, set)]
  y: i64
}
#[pymethods]
impl DummyPos {
  #[new]
  #[args(level = "0", room = "0", score = "0", x = "0", y = "0")]
  fn new(_py: Python, level: i64, room: i64, score: i64, x: i64, y: i64) -> Self {
    DummyPos {level, room, score, x, y }
  }
  pub fn set_tuple(&mut self, _py: Python, _arg: &PyAny) -> PyResult<i64> {
    Ok(0)
  }
  pub fn __repr__(&mut self, _py: Python) -> PyResult<String> {
    Ok(format!("Level={} Room={} Objects={} x={} y={}", self.level, self.room, self.score, self.x, self.y))
  }
  pub fn __getitem__(&self, py: Python, _str_arg: &PyAny) -> PyResult<PyObject> {
    // let item_arg = str_arg.extract()?;
    Ok(self.x.into_py(py))
  }
  pub fn __setitem__(&self, py: Python, _str_arg: &PyAny) -> PyResult<PyObject> {
    // let item_arg = str_arg.extract()?;
    Ok(self.x.into_py(py))
  }
  // pub fn __hash__(&self) -> PyResult<i64> {
  //   let mut s = DefaultHasher::new();
  //   self.hash(&mut s);
  //   Ok(s.finish() as i64)
  // }
  pub fn __getstate__(&mut self, py: Python) -> PyResult<PyObject> {
    let tuple = PyTuple::new(py, &[self.level, self.score, self.room, self.x, self.y]).to_object(py);
    Ok(tuple)
  }
  pub fn __setstate__(&mut self, _py: Python, arg: &PyAny) -> PyResult<i64> {
    let tuple: &PyTuple = arg.downcast()?;
    self.level = tuple.get_item(0)?.extract()?;
    self.score = tuple.get_item(1)?.extract()?;
    self.room = tuple.get_item(2)?.extract()?;
    self.x = tuple.get_item(3)?.extract()?;
    self.y = tuple.get_item(4)?.extract()?;
    Ok(0)
  }
}

pub fn rust_space_to_gym_space(py: Python, space: Space) -> PyResult<&PyAny> {
  let spaces_module = py.import("gym.spaces")?;
  match space {
    Space::Discrete(size) => {
      let discrete_space = spaces_module.getattr("Discrete")?;
      discrete_space.call(PyTuple::new(py, &[size]), None)
    },
    Space::BoxedWithRange(ranged_shape) => {
      let shape: Vec<i64> = ranged_shape.iter().map(|a|a.0).collect();
      let box_space = spaces_module.getattr("Box")?;
      let box_kwargs = [("shape", shape),].into_py_dict(py);
      box_space.call(PyTuple::new(py, &[-1, 1]), Some(box_kwargs))
    }
  }
}

#[pymethods]
impl ResettablePyGym {
  // TODO: Make DummyPos usable
  // pub fn get_pos(&mut self, py: Python) -> PyResult<PyObject> {
  //   let pos = DummyPos::new(py, 0, 0, 0, 0, 0);
  //   let pos_cell = PyCell::new(py, pos)?;
  //   let pos_obj = pos_cell.to_object(py);
  //   Ok(pos_obj)
  // }
  // #[staticmethod]
  // pub fn make_pos(py: Python, score_arg: &PyAny, pos_arg: &PyAny) -> PyResult<DummyPos> {
  //   let level: i64 = pos_arg.get_item("level")?.extract()?;
  //   let room: i64 = pos_arg.get_item("room")?.extract()?;
  //   let score: i64 = score_arg.extract()?;
  //   let x: i64 = pos_arg.get_item("x")?.extract()?;
  //   let y: i64 = pos_arg.get_item("y")?.extract()?;
  //   let pos = DummyPos::new(py, level, room, score, x, y);
  //   // let pos_cell = PyCell::new(py, pos)?;
  //   // let pos_obj = pos_cell.to_object(py);
  //   Ok(pos)
  // }
  pub fn get_pos(&mut self, py: Python) -> PyResult<PyObject> {
    let custom_module = py.import("goexplore_py.generic_cc_env")?;
    let custom_pos = custom_module.getattr("CCPosLevel")?;
    let pos_instance = custom_pos.call(PyTuple::empty(py), None)?;
    Ok(pos_instance.to_object(py))
  }
  // #[staticmethod]
  // pub fn make_pos(py: Python, score_arg: &PyAny, pos_arg: &PyDict) -> PyResult<PyObject> {
  //   let custom_module = py.import("goexplore_py.generic_cc_env")?;
  //   let custom_pos = custom_module.getattr("CCPosLevel")?;
  //   let pos_instance = custom_pos.call(PyTuple::empty(py), Some(pos_arg))?;
  //   pos_instance.set_item("score", score_arg)?;
  //   Ok(pos_instance.to_object(py))
  // }
  pub fn get_restore(&mut self, py: Python) -> PyResult<PyObject> {
    let dict: &PyDict = self.rust_env.get_restorable_state().into_py_dict(py);
    Ok(dict.to_object(py))
  }
  pub fn restore(&mut self, py: Python, arg: &PyAny) -> PyResult<PyObject> {
    let data_to_restore_opt: PyResult<HashMap<String, String>> = arg.extract();
    if let Ok(data_to_restore) = data_to_restore_opt {
      self.rust_env.restore_state(&data_to_restore);
    }
    // TODO: find out if we really need to return the full history here or if it doesn't matter
    Ok(arg.to_object(py))
  }
  pub fn reset(&mut self, py: Python) -> PyResult<PyObject> {
    let start = Instant::now();
    let reset_result = self.rust_env.reset();
    let reset_py_result = PyList::new(py, reset_result.into_raw_vec());
    let res = PyResult::Ok(reset_py_result.to_object(py));
    self.time_in_env += start.elapsed().as_nanos();
    res
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
    let observation = PyList::new(py, step_res.obs.into_raw_vec());
    let reward_tuple = PyTuple::new(py, &[step_res.reward]);
    let done_tuple = PyTuple::new(py, &[step_res.is_done]);
    let reward = reward_tuple.get_item(0).unwrap();
    let done = done_tuple.get_item(0).unwrap();
    let additional_info: &PyDict = [("empty", "info")].into_py_dict(py);
    let result = PyTuple::new(py, &[observation.as_ref(), reward, done, additional_info.as_ref()]);
    let res = PyResult::Ok(result.to_object(py));
    self.time_in_env += start.elapsed().as_nanos();
    res
  }
}

impl ResettablePyGym {
  pub fn new(py: Python, rust_env: Box<dyn ResettableGymEnvironment>) -> Self {
    let start = Instant::now();
    let action_space = rust_space_to_gym_space(py, rust_env.action_space()).unwrap();
    let action_space = action_space.to_object(py);
    let observation_space = rust_space_to_gym_space(py, rust_env.observation_space()).unwrap();
    let observation_space = observation_space.to_object(py);
    let spec = observation_space.to_object(py);
    //TODO: make the reward range defineable
    let b = PyList::new(py, &[-2, 2]);
    let reward_range = b.to_object(py);
    let rooms_b = PyList::empty(py);
    let rooms = rooms_b.to_object(py);
    let mut res = ResettablePyGym {
      action_space, observation_space, reward_range, spec, rust_env, rooms, time_in_env: 0
    };
    res.time_in_env = start.elapsed().as_nanos();
    res
  }
}