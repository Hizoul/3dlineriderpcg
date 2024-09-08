use super::VisualisableGymEnvironment;

pub type VisGymMaker = Box<dyn Fn() -> Box<dyn VisualisableGymEnvironment>>;

pub struct VisualisableGymRegistry {
  pub gyms: Vec<VisGymMaker>
}

impl Default for VisualisableGymRegistry {
  fn default() -> VisualisableGymRegistry {
    let reg = VisualisableGymRegistry::new();
    // #[cfg(feature = "env-control")] {
    //   use crate::env::control::{CartpoleEnv, PendulumEnv};
    //   reg.register_env(Box::new(|| {Box::new(CartpoleEnv::default())}));
    //   reg.register_env(Box::new(|| {Box::new(PendulumEnv::default())}));
    // }
    reg
  }
}

impl VisualisableGymRegistry {
  pub fn new() -> VisualisableGymRegistry {
    VisualisableGymRegistry {gyms: Vec::new()}
  }

  pub fn register_env(&mut self, env_maker: VisGymMaker) {
    let current_name = env_maker().get_name();
    let mut already_contained = false;
    for gym_maker in &self.gyms {
      if gym_maker().get_name() == current_name {
        already_contained = true;
      }
    }
    if !already_contained {
      self.gyms.push(env_maker);
    }
  }

  pub fn get_env(&self, env_name: &str) -> Option<Box<dyn VisualisableGymEnvironment>> {
    for gym_maker in &self.gyms {
      let gym = gym_maker();
      if gym.get_name() == env_name {
        return Some(gym);
      }
    }
    None
  }

  pub fn get_env_list(&self) -> Vec<String> {
    let mut names = Vec::with_capacity(self.gyms.len());
    for gym_maker in &self.gyms {
      names.push(gym_maker().get_name());
    }
    names
  }
}


#[cfg(test)]
pub mod test {
  use super::VisualisableGymRegistry;
  use crate::{env::zero_or_one::EnvZeroOrOne, replay::ReplayableGymEnvironment};

  #[test]
  fn env_registry_test() {
    let env = EnvZeroOrOne::default();
    let env_name = env.get_name();
    let mut registry = VisualisableGymRegistry::new();
    assert_eq!(true, registry.get_env(&env_name).is_none());
    registry.register_env(Box::new(|| {Box::new(EnvZeroOrOne::default())}));
    let found_env = registry.get_env(&env_name);
    assert_eq!(false, found_env.is_none());
  }
}
