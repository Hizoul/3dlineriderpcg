use crate::{gym::{Action, Observation, Space}, ReplayableGymEnvironment};
use std::collections::HashMap;

/**
 * A trait to connect a RL Algorithm to a Gym Environment
 * Supports ID, saving, loading and acting
 * todo: add reset
 **/
 pub trait RlAlgorithm {
  fn act(&mut self, obs: Observation) -> Action;
  fn set_observation_shape(&mut self, space: Space);
  fn save(&mut self, save_path: &str);
  fn load(&mut self, load_path: &str);
  fn get_hyperparams(&mut self) -> HashMap<String, String>;
  fn load_hyperparams(&mut self, config: &HashMap<String, String>);
  fn get_name(&self) -> String;
  fn reset(&mut self);
}

/**
 * A trait enabling the training of an RL Algorithm
 **/
pub trait SelfTrainingAlgo: RlAlgorithm {
  fn train_on_env(&mut self, rust_env: Box<dyn ReplayableGymEnvironment>, seed: Option<u64>, steps: usize);
}
