use crate::{
  gym::{GymEnvironment, Step, Observation, Action},
  space::Space,
  util::rng::{UniRng, RngType, get_rng_for_type, uni_gen_range_f64}
};
use ndarray::ArrayBase;
use std::f64::consts::PI;


fn angle_normalize(x: f64) -> f64 {
  ((x+PI) % (2.0*PI)) - PI
}
pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
  assert!(min <= max);
  if x < min {
    min
  } else if x > max {
    max
  } else {
    x
  }
}
pub struct PendulumEnv {
  rng: UniRng,
  rng_type: Option<RngType>,
  action_space: Space,
  observation_space: Space,
  max_speed: f64,
  max_torque: f64,
  dt: f64,
  g: f64,
  m: f64,
  l: f64,
  theta: f64,
  theta_dot: f64,
  used_seed: u64,
  step: usize,
  step_limit: usize
}

impl PendulumEnv {
  pub fn new(seed: Option<u64>, rng_type: Option<RngType>, g_opt: Option<f64>) -> PendulumEnv {
    let g = g_opt.unwrap_or(10.0);
    let (rng, used_seed) = get_rng_for_type(&rng_type, seed);
    let max_torque = 2.0;
    let max_speed = 8.0;
    let mut env = PendulumEnv {
      rng,
      rng_type: rng_type,
      used_seed,
      action_space: Space::BoxedWithRange(vec![1], vec![-max_torque], vec![max_torque]),
      observation_space: Space::BoxedWithRange(
        vec![3], vec![-1.0, -1.0, -max_speed], vec![1.0, 1.0, max_speed]
      ),
      max_speed,
      max_torque,
      dt: 0.05,
      g,
      m: 1.0,
      l: 1.0,
      theta: 0.0,
      theta_dot: 0.0,
      step_limit: 200,
      step: 0
    };
    env.reset_state();
    env
  }
  pub fn reset_state(&mut self) {
    self.theta = uni_gen_range_f64(&self.rng_type, &mut self.rng, -PI, PI);
    self.theta_dot = uni_gen_range_f64(&self.rng_type, &mut self.rng, -1.0 ,1.0);
    self.step = 0;
  }

  pub fn make_obs(&self) -> Observation {
    ArrayBase::from(vec![self.theta.cos(), self.theta.sin(), self.theta_dot]).into_dyn()
  }
}

impl Default for PendulumEnv {
  fn default() -> PendulumEnv {PendulumEnv::new(None, None, None)}
}

impl GymEnvironment for PendulumEnv {
  fn use_seed(&mut self, seed: u64) {
    let (new_rng, new_seed) = get_rng_for_type(&self.rng_type, Some(seed));
    self.rng = new_rng;
    self.used_seed = new_seed;
  }
  fn reset(&mut self) -> Observation {
    self.reset_state();
    self.make_obs()
  }

  fn step(&mut self, action: &Action) -> Step {
    let action_value = clamp(action[0], -self.max_torque, self.max_torque);

    let costs = angle_normalize(self.theta).powi(2) + 0.1*self.theta_dot.powi(2) + 0.001 * action_value.powi(2);

    let new_theta_dot = self.theta_dot + (-3.0 * self.g / (2.0 * self.l) * (self.theta + PI).sin() + 3.0 / (self.m * self.l.powi(2)) * action_value) * self.dt;
    self.theta = self.theta + new_theta_dot * self.dt;
    self.theta_dot = clamp(new_theta_dot, -self.max_speed, self.max_speed);

    let reward = -costs;
    self.step += 1;
    let is_done = self.step >= self.step_limit;
    Step {
      obs: self.make_obs(),
      reward,
      is_done,
      action: action.clone()
    }
  }

  fn action_space(&self) -> Space {
    self.action_space.clone()
  }

  fn observation_space(&self) -> Space {
    self.observation_space.clone()
  }
}


#[cfg(feature = "eval")]
use crate::ReplayableGymEnvironment;
#[cfg(feature = "eval")]
use std::collections::HashMap;
#[cfg(feature = "eval")]
impl ReplayableGymEnvironment for PendulumEnv {
  fn get_used_seed(&mut self) -> u64 {self.used_seed}
  fn get_config(&mut self) -> HashMap<String, String> {HashMap::new()}
  fn load_config(&mut self, _config: &HashMap<String, String>) {}
  fn get_name(&self) -> String {"rusty-Pendulum-v1".to_owned()}
  fn finalize(&mut self, _algo_name: &str, _eval_run_id: &str) {}
}

#[cfg(feature = "python")]
#[cfg(test)]
pub mod test {
  // use crate::{GymEnvironment, env::python::PythonToRustGym, util::rng::{RngType}};
  use pyo3::prelude::*;
  // use ndarray::ArrayBase;
  // use super::PendulumEnv;
  #[test]
  fn python_to_rust_test() -> Result<(), PyErr> {
    // let seed_to_compare = 4242;//rng.next_u64();
    // let mut py_pendulum = PythonToRustGym::from_str("Pendulum-v0", Some(seed_to_compare))?;
    // py_pendulum.use_seed(seed_to_compare);
    // py_pendulum.reset();
    // let mut rust_cart = PendulumEnv::new(Some(seed_to_compare), Some(RngType::Mt19937), None);
    // let mut rust_cart_diff = PendulumEnv::new(Some(seed_to_compare), Some(RngType::Pcg64Mcg), None);

    // let action = ArrayBase::from(vec![1.0]).into_dyn();
    // let mut done = false;
    // while !done {
    //   let py_res = py_pendulum.step(&action);
    //   let rust_res = rust_cart.step(&action);
    //   let rust_res_diff = rust_cart_diff.step(&action);
    //   assert_eq!(py_res, rust_res);
    //   assert_ne!(rust_res_diff, rust_res);
    //   done = rust_res.is_done;
    // }

    Ok(())
  }
}