use pyo3::prelude::*;
use pyo3::types::{PyTuple, PyList, PyDict};

use crate::{Space};

pub fn make_numpy_array(py: Python, to_convert: Vec<f64>) -> PyResult<&PyAny> {
  let numpy_module = py.import("numpy")?;
  let nparray = numpy_module.getattr("array")?;
  nparray.call(PyTuple::new(py, &[to_convert]), None)
}

pub fn reshape_numpy_array(py: Python, to_convert: Vec<f64>, shape: Vec<i64>) -> PyResult<&PyAny> {
  let numpy_module = py.import("numpy")?;
  let reshape = numpy_module.getattr("reshape")?;
  let to_convert_py = PyList::new(py, to_convert);
  let shape_py = PyList::new(py, shape);
  reshape.call(PyTuple::new(py, &[to_convert_py, shape_py]), None)
}

pub fn get_obs_shape(space: Space) -> Option<Vec<i64>> {
  let mut obs_shape: Option<Vec<i64>> = None;
  match space {
    Space::BoxedWithRange(shape, _, _) => {
      obs_shape = Some(shape.clone());
    },
    Space::BoxedWithoutRange(shape) => {
      obs_shape = Some(shape.clone());
    },
    _ => {}
  }
  obs_shape
}

pub fn rust_space_to_gym_space(py: Python, space: Space) -> PyResult<&PyAny> {
  let spaces_module = py.import("gymnasium.spaces")?;
  match space {
    Space::Discrete(size) => {
      let discrete_space = spaces_module.getattr("Discrete")?;
      discrete_space.call(PyTuple::new(py, &[size]), None)
    },
    Space::BoxedWithRange(shape, low, high) => {
      // let low = ranged_shape[0].1[0].0;
      // let high = ranged_shape[0].1[0].1;
      let box_space = spaces_module.getattr("Box")?;
      let box_kwargs = PyDict::new(py);
      box_kwargs.set_item("shape", shape.clone())?;
      box_kwargs.set_item("low", reshape_numpy_array(py, low, shape.clone())?)?;
      box_kwargs.set_item("high", reshape_numpy_array(py, high, shape)?)?;
      // let box_kwargs = [("shape", shape)].into_py_dict(py); //[("shape", shape),("low", low), ("high", high)].into_py_dict(py);
      box_space.call(PyTuple::empty(py), Some(box_kwargs))
    },
    Space::BoxedWithoutRange(shape) => {
      let box_space = spaces_module.getattr("Box")?;
      let box_kwargs = PyDict::new(py);
      box_kwargs.set_item("shape", shape)?;
      box_space.call(PyTuple::empty(py), Some(box_kwargs))
    }
  }
}