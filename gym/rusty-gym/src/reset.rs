use crate::{gym::{GymEnvironment}};
use std::collections::HashMap;

pub trait ResettableGymEnvironment: GymEnvironment {
  fn restore_state(&mut self, state: &HashMap<String, String>);
  fn get_restorable_state(&mut self) -> HashMap<String, String>;
}


