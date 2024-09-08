use crate::{space::Space,
  gym::{GymEnvironment, Reward, RewardVector, Action, Observation, Step}, RUNTYPE_TRAINING, RunData, RUNTYPE_EVAL};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Instant;
use compressed_vec::CompressedVec;
use xp_tools::{rng::rng_with_random_seed, save_cbor_and_flate_to_path, get_env_variable, generate_id};

pub const ENV_REPLAY_PATH: &str = "TLF_REPLAY_PATH";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpisodeData {
  pub log: Vec<Action>,
  pub seed: u64,
  pub algorithm_hyperparams: Option<HashMap<String, String>>,
  pub env_params: Option<HashMap<String, String>>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnrichedEpisodeData {
  pub log: Vec<Action>,
  pub seed: u64,
  pub rewards: RewardVector,
  pub task_completion: f64,
  pub episode_nr: usize,
  pub algorithm_hyperparams: Option<HashMap<String, String>>,
  pub env_params: Option<HashMap<String, String>>,
  pub additional_info: Option<HashMap<String, String>>
}

impl EpisodeData {
  pub fn new(seed: u64, log: Vec<Action>) -> EpisodeData {
    EpisodeData {
      seed, log, algorithm_hyperparams: None, env_params: None
    }
  }
}

impl EnrichedEpisodeData {
  pub fn to_regular_episode(&self) -> EpisodeData {
    let mut ep = EpisodeData::new(self.seed, self.log.clone());
    ep.algorithm_hyperparams = self.algorithm_hyperparams.clone();
    ep.env_params = self.env_params.clone();
    ep
  }
}

pub type Episodes = Vec<EpisodeData>;
pub type EnrichedEpisodes = Vec<EnrichedEpisodeData>;

pub trait ReplayableGymEnvironment: GymEnvironment {
  fn get_used_seed(&mut self) -> u64;
  fn get_config(&mut self) -> HashMap<String, String>;
  fn load_config(&mut self, config: &HashMap<String, String>);
  fn get_name(&self) -> String;
  fn finalize(&mut self, algo_name: &str, eval_run_id: &str);
}

use std::sync::{Arc, Mutex};

pub struct GymRecorder {
  pub data: Arc<Mutex<CompressedVec<EpisodeData>>>,
  pub episode_actions: Vec<Action>,
  original_env: Box<dyn ReplayableGymEnvironment>,
  env_seed: u64,
  start: Instant,
  pub timed: bool,
  pub track_env_config: bool,
  pub was_done: bool,
  pub manage_seed: bool,
  pub run_id: String
}

impl GymRecorder {
  pub fn new(original_env: Box<dyn ReplayableGymEnvironment>, id_opt: Option<String>) -> GymRecorder {
    let manage_seed = true; // TODO: env_name.contains("CartPole");
    let data = Arc::new(Mutex::new(CompressedVec::new()));
    let run_id = id_opt.unwrap_or(generate_id());
    GymRecorder {
      data,
      episode_actions: Vec::with_capacity(1000),
      original_env,
      env_seed: 0,
      start: Instant::now(),
      timed: false,
      track_env_config: false,
      was_done: false,
      run_id,
      manage_seed
    }
  }
}

use rand::RngCore;
impl GymEnvironment for GymRecorder {
  fn action_space(&self) -> Space {self.original_env.action_space()}
  fn observation_space(&self) -> Space {self.original_env.observation_space()}
  fn step(&mut self, action: &Action) -> Step {
    if !self.was_done {
      self.episode_actions.push(action.clone());
    } else {
      println!("CALLED AFTER DONE!");
    }
    let step_data = self.original_env.step(action);
    if step_data.is_done {
      self.was_done = true;
    }
    step_data
  }
  fn reset(&mut self) -> Observation {
    self.was_done = false;
    if self.episode_actions.len() > 0 {
      let env_seed = self.get_used_seed();
      let mut data = self.data.lock().unwrap();
      let mut new_episode = EpisodeData::new(env_seed, self.episode_actions.clone());
      if self.track_env_config {
        new_episode.env_params = Some(self.original_env.get_config());
      }
      data.push(new_episode);
      self.episode_actions.clear();
    }
    let obs = if self.manage_seed {
      self.use_seed(rng_with_random_seed().next_u64());
      self.original_env.reset()
    } else {
      let o = self.original_env.reset();
      self.env_seed = self.original_env.get_used_seed();
      o
    };
    if self.timed {
      println!("Episode took {}ms", self.start.elapsed().as_millis());
      self.start = Instant::now();
    }
    obs
  }
  fn use_seed(&mut self, seed: u64) {
    self.env_seed = seed;
    self.original_env.use_seed(seed)
  }
}

impl ReplayableGymEnvironment for GymRecorder {
  fn get_used_seed(&mut self) -> u64 {
    if self.manage_seed {
      self.env_seed
    } else {
      self.original_env.get_used_seed()
    } 
  }
  fn get_config(&mut self) -> HashMap<String, String> {
    let mut conf = self.original_env.get_config();
    conf.insert("run_id".to_owned(), self.run_id.clone());
    conf
  }
  fn load_config(&mut self, config: &HashMap<String, String>) {self.original_env.load_config(config)}
  fn get_name(&self) -> String {self.original_env.get_name()}
  fn finalize(&mut self, algo_name: &str, eval_run_id: &str) {
    self.original_env.finalize(algo_name, eval_run_id);
    let config = self.get_config();
    let mut unlocked_eps = self.data.lock().unwrap();
    unlocked_eps.finalize();
    let env_name = self.get_name();

    let result_dir = get_env_variable("TLF_REPLAY_PATH").unwrap_or("trl-experiments".to_owned());
    let (run_type, eval_id) = {
      if eval_run_id.len() == 0 {
        (RUNTYPE_TRAINING, None)
      } else {
        (RUNTYPE_EVAL, Some(eval_run_id.to_owned()))
      }
    };
    let new_run_data = RunData::new(run_type, env_name.clone(), algo_name.to_owned(), unlocked_eps.clone(), config, None, 0, Some(self.run_id.clone()), eval_id);
    #[cfg(not(target_arch = "wasm32"))]
    {
      let algo_dir = format!("{}/{}/{}", result_dir, env_name, algo_name);
      std::fs::create_dir_all(&algo_dir).expect(&format!("Able to create needed intermediete directories for path {}", &algo_dir));
      save_cbor_and_flate_to_path(format!("{}/{}.tlr", algo_dir, self.run_id).as_str(), &new_run_data);
    }
  }
}

pub fn episode_to_reward_vec_r(env: &mut Box<dyn ReplayableGymEnvironment>, episode: &EpisodeData) -> Vec<Reward> {
  let mut rewards = Vec::with_capacity(episode.log.len());
  env.use_seed(episode.seed);
  env.reset();
  for action in episode.log.iter() {
    let res = env.step(action);
    rewards.push(res.reward);
  }
  rewards
}


#[cfg(test)]
pub mod test {
  use crate::{GymEnvironment, Step, ReplayableGymEnvironment, Observation, env::control::CartpoleEnv, util::rng::RngType};
  use ndarray::ArrayBase;
  use super::GymRecorder;

  fn collect_episode(env: &mut GymRecorder, seed: Option<u64>) -> (u64, Vec<Step>) {
    let mut original_observations: Vec<Observation> = Vec::new();
    let (used_seed, initial_obs) = if let Some(se) = seed {
      env.manage_seed = false;
      env.use_seed(se);
      let obs = env.reset();
      (se, obs)
    } else {
      env.manage_seed = true;
      let obs = env.reset();
      (env.get_used_seed(), obs)
    };
    original_observations.push(initial_obs);
    let mut original_steps: Vec<Step> = Vec::new();

    let action = ArrayBase::from(vec![1.0]).into_dyn();
    let mut done = false;
    while !done {
      let rust_res = env.step(&action);
      done = rust_res.is_done;
      original_steps.push(rust_res);
    }
    (used_seed, original_steps)
  }

  #[test]
  fn replay_equal() {
    let rust_cart = CartpoleEnv::new(None, Some(RngType::Mt19937));
    let mut recorder = GymRecorder::new(Box::new(rust_cart), None);
    let orig = collect_episode(&mut recorder, None);
    for _ in 0..100 {
      let other = collect_episode(&mut recorder, Some(orig.0));
      assert_eq!(orig.1[0], other.1[0]);
    }
  }
}