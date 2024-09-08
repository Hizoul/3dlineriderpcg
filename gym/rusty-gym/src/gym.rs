pub use crate::space::Space;
use ndarray::ArrayD;

pub type Observation = ArrayD<f64>;
pub type Action = ArrayD<f64>;
pub type Reward = f64;
pub type RewardVector = Vec<Reward>;
pub type EpisodeRewards = Vec<RewardVector>;

#[derive(Debug, Clone, PartialEq)]
pub struct Step {
  pub obs: Observation,
  pub action: Action,
  pub reward: f64,
  pub is_done: bool,
}

pub trait GymEnvironment {
  fn action_space(&self) -> Space;
  fn observation_space(&self) -> Space;
  fn step(&mut self, action: &Action) -> Step;
  fn reset(&mut self) -> Observation;
  fn use_seed(&mut self, seed: u64);
}