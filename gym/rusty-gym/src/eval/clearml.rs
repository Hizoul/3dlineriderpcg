use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyTuple, PyList, PyDict, PyString};
pub fn clearml_init_task(project_name: &str, task_name: &str) -> PyObject {
  let gil = Python::with_gil();
  let py = gil.python();
  let locals = PyDict::new(py);
  let code_to_run = format!(r#"from clearml import Task
current_task = Task.create(project_name="{}", task_name="{}")"#, project_name, task_name);
  py.run(&code_to_run, None, Some(locals)).expect("Can create clearml task object");
  locals.get_item("current_task").expect("Can get clearml task ref").to_object(py)
}

pub fn clearml_mark_started(task_ref: &PyObject) {
  let gil = Python::with_gil();
  let py = gil.python();
  let locals = PyDict::new(py);
  locals.set_item("current_task", task_ref).expect("Can set item of PyDict");
  let code_to_run = "current_task.mark_started()";
  py.run(&code_to_run, None, Some(locals)).expect("Can mark clearml task as started");
}

pub fn clearml_mark_completed(task_ref: &PyObject) {
  let gil = Python::with_gil();
  let py = gil.python();
  let locals = PyDict::new(py);
  locals.set_item("current_task", task_ref).expect("Can set item of PyDict");
  let code_to_run = r#"logger = current_task.get_logger()
logger.flush()
current_task.mark_completed()"#;
  py.run(&code_to_run, None, Some(locals)).expect("Can mark clearml task as completed");
}

pub fn clearml_set_parameters(task_ref: &PyObject, parameters: &PyObject) {
  let gil = Python::with_gil();
  let py = gil.python();
  let locals = PyDict::new(py);
  locals.set_item("current_task", task_ref).expect("Can set item of PyDict");
  locals.set_item("parameters", parameters).expect("Can set item of PyDict");
  let code_to_run = "current_task.set_parameters(parameters)";
  py.run(&code_to_run, None, Some(locals)).expect("Can set task parameters");
}

pub fn clearml_set_parameter(task_ref: &PyObject, parameter_name: &str, parameters: &PyObject) {
  let gil = Python::with_gil();
  let py = gil.python();
  let locals = PyDict::new(py);
  locals.set_item("current_task", task_ref).expect("Can set item of PyDict");
  locals.set_item("parameters", parameters).expect("Can set item of PyDict");
  let code_to_run = format!("current_task.set_parameter(name='{}', value=parameters)", parameter_name);
  py.run(&code_to_run, None, Some(locals)).expect("Can set parameter");
}

pub fn clearml_connect_configuration(task_ref: &PyObject, parameter_name: &str, parameters: &PyObject) {
  let gil = Python::with_gil();
  let py = gil.python();
  let locals = PyDict::new(py);
  locals.set_item("current_task", task_ref).expect("Can set item of PyDict");
  locals.set_item("parameters", parameters).expect("Can set item of PyDict");
  let code_to_run = format!("current_task.connect_configuration(name='{}', configuration=parameters)", parameter_name);
  py.run(&code_to_run, None, Some(locals)).expect("Can set parameter");
}

pub fn clearml_report_scalar(task_ref: &PyObject, title: &str, series: &str, value: f64, iteration: i64) {
  let gil = Python::with_gil();
  let py = gil.python();
  let locals = PyDict::new(py);
  locals.set_item("current_task", task_ref).expect("Can set item of PyDict");
  let code_to_run = format!(r#"logger = current_task.get_logger()
logger.report_scalar("{}", "{}", value={}, iteration={})"#, title, series, value, iteration);
  py.run(&code_to_run, None, Some(locals)).expect("Can report scalar");
}

#[cfg(test)]
pub mod test {
  use super::*;
  use std::collections::HashMap;
  #[test]
  fn test_clearml_logging() {
    pyo3::prepare_freethreaded_python();
    let task = clearml_init_task("unittest", "unittest");
    clearml_mark_started(&task);
    let mut config: HashMap<String, String> = HashMap::new();
    config.insert("goal_size".to_string(), "3.5".to_string());
    config.insert("skip_collision_check_on_last_x_pieces".to_string(), "9999".to_string());
    config.insert("simulation_steps".to_string(), ((1000/80) * 600).to_string());

    let dictobj = {
      let gil = Python::with_gil();
      let py = gil.python();
      let dict = config.into_py_dict(py);
      dict.to_object(py)
    };
    clearml_set_parameters(&task, &dictobj);
    clearml_set_parameter(&task, "env", &dictobj);
    clearml_connect_configuration(&task, "env", &dictobj);
    for i in 0..10 {
      clearml_report_scalar(&task, "mymetric", "myseries", i as f64, i);
    }
    clearml_mark_completed(&task);
  }
}