#[cfg(feature = "vis")]
pub mod plot;
use crate::{algo::SelfTrainingAlgo, GymRecorder, ReplayableGymEnvironment, GymEnvironment, EpisodeData, EnrichedEpisodeData, Action, RewardVector, Reward};
use xp_tools::{fs::{async_load_cbor_and_flate_file, create_dir_if_it_doesnt_exist}, id::generate_id, http::{read_url_with_post, read_url}};
#[cfg(not(target_arch = "wasm32"))]
use xp_tools::{fs::save_cbor_and_flate_to_path, load_cbor_and_flate_file, save_json_to_path};
use serde::{Serialize, Deserialize};
use std::time::Instant;
use compressed_vec::CompressedVec;

#[cfg(feature = "vis")]
use crate::eval::plot::reward_err_graph;

use std::collections::HashMap;

pub const RUNTYPE_TRAINING: u8 = 2;
pub const RUNTYPE_EVAL: u8 = 3;
pub const RUNTYPES: [u8; 2] = [RUNTYPE_TRAINING, RUNTYPE_EVAL];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunData {
  pub episodes: CompressedVec<EpisodeData>,
  pub algo: String,
  pub env: String,
  pub run_type: u8,
  pub uid: String,
  pub reuses: Option<String>,
  pub is_eval_of: Option<String>,
  pub hyperparams: Option<HashMap<String, String>>,
  pub env_config: HashMap<String, String>,
  pub time_needed: u64,
  pub reward_per_episode: Option<Vec<Reward>>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunDataEnriched {
  pub episodes: CompressedVec<EnrichedEpisodeData>,
  pub algo: String,
  pub env: String,
  pub run_type: u8,
  pub uid: String,
  pub reuses: Option<String>,
  pub is_eval_of: Option<String>,
  pub hyperparams: Option<HashMap<String, String>>,
  pub env_config: HashMap<String, String>,
  pub time_needed: u64,
  pub reward_per_episode: Option<Vec<Reward>>
}

impl RunData {
  pub fn new(run_type: u8, env: String, algo: String, episodes: CompressedVec<EpisodeData>, env_config: HashMap<String, String>, hyperparams: Option<HashMap<String, String>>, time_needed: u64, run_id: Option<String>, is_eval_of: Option<String>) -> RunData {
    RunData {
      run_type, algo, env,
      episodes, uid: run_id.unwrap_or_else(generate_id),
      reuses: None, reward_per_episode: None, env_config, is_eval_of, hyperparams,
      time_needed
    }
  }
}

impl RunDataEnriched {
  pub fn new(original_run: RunData, episodes: CompressedVec<EnrichedEpisodeData>) -> RunDataEnriched {
    RunDataEnriched {
      algo: original_run.algo,
      env: original_run.env,
      run_type: original_run.run_type,
      uid: original_run.uid,
      reuses: original_run.reuses,
      env_config: original_run.env_config,
      episodes, is_eval_of: original_run.is_eval_of,
      hyperparams: original_run.hyperparams,
      time_needed: original_run.time_needed,
      reward_per_episode: original_run.reward_per_episode
    }
  }
}
//TODO Exchange
fn avg(list: &[f64]) -> f64 {
  let sum: f64 = Iterator::sum(list.iter());
  sum / (list.len() as f64)
}
pub type EvaluationData = Vec<RunData>;

pub type EnvironmentMaker = Box<dyn Fn(&HashMap<String, String>) -> Box<dyn ReplayableGymEnvironment> + Send + Sync>;

#[allow(dead_code)] // TODO: match algo params impl
pub struct TransferConfigDetail {
  pub env: HashMap<String, String>,
  pub algo: HashMap<String, String>,
  /**
   * The transfer experiments will only look for experiments with the same algo name and the same env config.
   * If you want to ensure that transfer happens only if the algorithm had the same hyperparameters then set this to true.
   */
  has_to_match_algo_params_as_well: bool
}

impl TransferConfigDetail {
  pub fn new(env: HashMap<String, String>, algo: HashMap<String, String>) -> TransferConfigDetail {
    TransferConfigDetail {
      env, algo, has_to_match_algo_params_as_well: false
    }
  }
}

pub type TransferConfig = (TransferConfigDetail, TransferConfigDetail);

pub struct RlExperimentHelper {
  pub algorithms: Vec<Box<dyn SelfTrainingAlgo>>,
  pub environments: Vec<EnvironmentMaker>,
  pub result_dir: String,
  pub runs_per_env: usize,
  pub steps_per_env: usize,
  pub eval_episode_amount: usize,
  pub run_data: EvaluationData
}

impl Default for RlExperimentHelper {
  fn default() -> RlExperimentHelper {
    Self::new()
  }
}

pub fn get_run_amount(run_data: &[RunData], env_name: &str, algo_name: &str, run_type: u8, env_config: &HashMap<String, String>, has_reuse: bool) -> usize {
  let filtered: Vec<&RunData> = run_data.iter().filter(|run| {
    (has_reuse && run.reuses.is_some() || !has_reuse && run.reuses.is_none()) && run.env == env_name && run.algo == algo_name && run.run_type == run_type && &run.env_config == env_config
  }).collect();
  filtered.len()
}

pub fn find_run_without_eval(run_data: &[RunData], env_name: &str, algo_name: &str, run_type: u8, env_config: &HashMap<String, String>) -> Option<RunData> {
  let find_res = run_data.iter().find(|run| {
    if run.env == env_name && run.algo == algo_name && run.run_type == run_type && &run.env_config == env_config {
      run_data.iter().any(|secrun| { secrun.is_eval_of.is_some() && secrun.is_eval_of.as_ref().unwrap() == &run.uid})
    } else {
      false
    }
  });
  if let Some(run) = find_res {
    Some(run.clone())
  } else { None }
}

impl RlExperimentHelper {
  pub fn new() -> RlExperimentHelper {
    let algorithms: Vec<Box<dyn SelfTrainingAlgo>> = Vec::new();
    let environments: Vec<EnvironmentMaker> = Vec::new();
    RlExperimentHelper::with_config("/tmp/worldeval".to_owned(), algorithms, environments)
  }
  pub fn with_config(result_dir: String, algorithms: Vec<Box<dyn SelfTrainingAlgo>>, environments: Vec<EnvironmentMaker>) -> RlExperimentHelper {
    let mut created_eval = RlExperimentHelper {
      algorithms,
      environments,
      result_dir,
      runs_per_env: 2,
      steps_per_env: 1000,
      eval_episode_amount: 10,
      run_data: Vec::with_capacity(100000)
    };
    created_eval.load();
    created_eval
  }
  pub fn save(&self) {
    #[cfg(not(target_arch = "wasm32"))]
    {
      save_cbor_and_flate_to_path(format!("{}/run_data.bin", self.result_dir), &self.run_data);
    }
  }
  pub fn load(&mut self) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if metadata(format!("{}/run_data.bin", self.result_dir)).is_ok() {
          println!("Found training data to load in {}", self.result_dir);
          self.run_data = load_cbor_and_flate_file(format!("{}/run_data.bin", self.result_dir));
        }
    }
  }
  pub async fn load_cross_platform(&mut self, path_to_load: &str) {
    self.run_data = async_load_cbor_and_flate_file(path_to_load).await;
  }
  pub async fn load_replay_from_enriched_url(&mut self, path_to_load: &str, episode: usize) -> (Box<dyn ReplayableGymEnvironment>, u64, Vec<Action>) {
    let mut run_data: RunDataEnriched = async_load_cbor_and_flate_file(path_to_load).await;
    let episode_data = run_data.episodes.index(episode).clone();
    let seed = episode_data.seed;
    let episode_config = run_data.env_config.clone();
    let env_init = self.environments.iter_mut().find(|env_init| {
      let env = (*env_init)(&episode_config);
      env.get_name() == run_data.env
    }).unwrap();
    let mut env = env_init(&episode_config);
    env.use_seed(seed);
    env.reset();
    (env, seed, episode_data.log)
  }
  pub async fn load_replay(&mut self, env_name: &str, algo_name: &str, run_type: u8, occurence: usize, episode: usize) -> (Box<dyn ReplayableGymEnvironment>, u64, Vec<Action>) {
    let mut filtered: Vec<&mut RunData> = self.run_data.iter_mut().filter(|run| {
      run.env == env_name && run.algo == algo_name && run.run_type == run_type
    }).collect();
    let run_data = &mut filtered[occurence];
    let episode_data = run_data.episodes.index(episode).clone();
    let seed = episode_data.seed;
    let episode_config = run_data.env_config.clone();
    let env_init = self.environments.iter_mut().find(|env_init| {
      let env = (*env_init)(&episode_config);
      env.get_name() == run_data.env
    }).unwrap();
    let mut env = env_init(&episode_config);
    env.use_seed(seed);
    env.reset();
    (env, seed, episode_data.log)
  }
  #[cfg(feature = "vis")]
  pub fn draw_graphs(&mut self) { // TODO: this should iterate over run_data instead of the way it is working now
    let envs = self.environments.iter_mut();
    for env_m in envs {
      let env = env_m(&HashMap::new()); // TODO: what config?
      let env_name = env.get_name();
      let algos = self.algorithms.iter_mut();
      for algo in algos {
        let algo_name = algo.get_name();
        let ea_combo = format!("{}_{}", env_name, algo_name);
        for graph_run_type in RUNTYPES.iter() {
          let relevant_runs: Vec<&mut RunData> = self.run_data.iter_mut().filter(|run| {
            run.env == env_name && run.algo == algo_name && &run.run_type == graph_run_type
          }).collect();
          let mut to_graph_avg: Vec<RewardVector> = Vec::with_capacity(1000);
          for run in relevant_runs {
            let mut enriched_episodes: CompressedVec<EnrichedEpisodeData> = CompressedVec::new();
            for i in 0..run.episodes.len() {
              enriched_episodes.push(enrich_episode(run, i, &env_m));
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
              let algo_dir = format!("{}/{}/{}", self.result_dir, env_name, algo_name);
              save_cbor_and_flate_to_path(format!("{}/{}.tlrx", algo_dir, run.uid).as_str(), &RunDataEnriched::new(run.clone(), enriched_episodes.clone()));
            }
            let mut avg_per_episode = Vec::new();
            for idx in 0..enriched_episodes.len() {
              let enriched_episode = enriched_episodes.index(idx);
              avg_per_episode.push(
                avg(enriched_episode.rewards.as_slice())
              );
            }
            to_graph_avg.push(avg_per_episode);
          }
          let graph_name = if graph_run_type == &RUNTYPE_TRAINING {"training"} else {"eval"};
          println!("BEGINNING TGO DRAW {}", format!("{}/{}_{}.png", self.result_dir, ea_combo, graph_name));
          reward_err_graph(format!("{}/{}_{}.png", self.result_dir, ea_combo, graph_name).as_str(), algo_name.as_str(), &to_graph_avg, None);
          // reward_err_graph(format!("{}/{}_{}_zoom.png", self.result_dir, ea_combo, graph_name).as_str(), algo_name.as_str(), &to_graph_avg, Some(6));
        }
      }
    }
  }
  pub fn run_experiment_for_config(env_maker: &mut EnvironmentMaker, algo: &mut Box<dyn SelfTrainingAlgo>, env_config: &HashMap<String, String>, algo_config_opt: Option<HashMap<String, String>>, current_run: usize, transfer_from_this_config: Option<HashMap<String, String>>, result_dir: &str, run_data: &mut EvaluationData, runs_per_env: usize, steps_per_env: usize, eval_episode_amount: usize) {
    let boxed_env: Box<dyn ReplayableGymEnvironment> = env_maker(&env_config);
    let env_name = boxed_env.get_name();
    let algo_name = algo.get_name();
    let run_number_training = get_run_amount(&run_data, &env_name, &algo_name, RUNTYPE_TRAINING, &env_config, transfer_from_this_config.is_some());
    let mut eval_this_run: Option<String> = None;
    let mut reuses: Option<String> = None;
    if runs_per_env > run_number_training {
      let run_id = generate_id();
      eval_this_run = Some(run_id.clone());
      println!("Training {} on {}. Run #{}/{}", algo_name, env_name, current_run + 1, runs_per_env);
      let recording_env = GymRecorder::new(boxed_env, None);
      let episode_data = recording_env.data.clone();
      if let Some(load_from_this_config) = &transfer_from_this_config {
        let existing_trainings: Vec<&RunData> = run_data.iter().filter(|run| {
          run.reuses.is_none() && run.env == env_name && run.algo == algo_name && run.run_type == RUNTYPE_TRAINING && &run.env_config == load_from_this_config
        }).collect();
        'FIND_REUSABLE_RUN: for existing_training in &existing_trainings {
          let existing_transfer_for_this_uid: Vec<&RunData> = run_data.iter().filter(|run| {
            if let Some(run_reuses) = &run.reuses {
              return run_reuses == &existing_training.uid
            }
            false
          }).collect();
          if existing_transfer_for_this_uid.len() == 0 { // TODO: what about multiple transfer runs?
            reuses = Some(existing_training.uid.clone());
            break 'FIND_REUSABLE_RUN;
          }
        }
        if reuses.is_none() && existing_trainings.len() > 0 {
          reuses = Some(existing_trainings[0].uid.clone());
        }
        if let Some(reuse_this_id) = &reuses {
          let algo_dir = format!("{}/{}/{}", result_dir, env_name, algo_name);
          let load_from = format!("{}/{}_weights", algo_dir, reuse_this_id);
          algo.load(&load_from);
        } else {
          panic!("Could not find a re-usable training to transfer from for config {:?}", load_from_this_config);
        }
      } else {
        algo.reset();
      }
      if let Some(algo_config) = algo_config_opt {
        algo.load_hyperparams(&algo_config);
      }
      let start = Instant::now();
      algo.train_on_env(Box::new(recording_env), None, steps_per_env);
      let time_needed = start.elapsed().as_millis();
      let mut episodes = {
        let unlocked_eps = episode_data.lock().unwrap();
        unlocked_eps.clone()
      };
      episodes.finalize();
      let mut new_run_data = RunData::new(RUNTYPE_TRAINING, env_name.clone(), algo_name.clone(), episodes, env_config.clone(), Some(algo.get_hyperparams()), time_needed as u64, Some(run_id.clone()), None);
      new_run_data.reuses = reuses.clone();
      #[cfg(not(target_arch = "wasm32"))]
      {
        let algo_dir = format!("{}/{}/{}", result_dir, env_name, algo_name);
        save_cbor_and_flate_to_path(format!("{}/{}.tlr", algo_dir, run_id).as_str(), &new_run_data);
        algo.save(format!("{}/{}_weights", algo_dir, run_id).as_str());
      }
      run_data.push(new_run_data);
      #[cfg(not(target_arch = "wasm32"))]
      {
        save_cbor_and_flate_to_path(format!("{}/run_data.bin", result_dir), &run_data);
      }
    } else {
      let run_without_eval = find_run_without_eval(&run_data, &env_name, &algo_name, RUNTYPE_TRAINING, &env_config);
      if let Some(run) = run_without_eval {
        eval_this_run = Some(run.uid.clone());
      }
    }
    if let Some(to_eval_id) = eval_this_run {
      println!("Evaluating {} on {}. Run #{}/{}", algo_name, env_name, current_run + 1, runs_per_env);
      #[cfg(not(target_arch = "wasm32"))]
      {
        let algo_dir = format!("{}/{}/{}", result_dir, env_name, algo_name);
        algo.load(format!("{}/{}_weights", algo_dir, to_eval_id).as_str());
      }
      let eval_run_id = generate_id();
      let boxed_env: Box<dyn ReplayableGymEnvironment> = env_maker(&env_config);
      let mut recording_env = GymRecorder::new(boxed_env, None);
      let episode_data = recording_env.data.clone();
      let mut obs = recording_env.reset();
      let mut current_episode = 0;
      let start = Instant::now();
      algo.set_observation_shape(recording_env.observation_space());
      while current_episode < eval_episode_amount {
        let action = algo.act(obs);
        let step = recording_env.step(&action);
        obs = if step.is_done {
          current_episode += 1;
          recording_env.reset()
        } else {
          step.obs
        };
      }
      let time_needed = start.elapsed().as_millis();
      let mut episodes = {
        let unlocked_eps = episode_data.lock().unwrap();
        unlocked_eps.clone()
      };
      episodes.finalize();
      let mut new_run_data = RunData::new(RUNTYPE_EVAL, env_name.clone(), algo_name.clone(), episodes, env_config.clone(), None, time_needed as u64, Some(eval_run_id.clone()), Some(to_eval_id));
      new_run_data.reuses = reuses;
      #[cfg(not(target_arch = "wasm32"))]
      {
        let algo_dir = format!("{}/{}/{}", result_dir, env_name, algo_name);
        save_cbor_and_flate_to_path(format!("{}/{}.tlr", algo_dir, eval_run_id).as_str(), &new_run_data);
        algo.save(format!("{}/{}_weights", algo_dir, eval_run_id).as_str());
      }
      run_data.push(new_run_data);
      #[cfg(not(target_arch = "wasm32"))]
      {
        save_cbor_and_flate_to_path(format!("{}/run_data.bin", result_dir), &run_data);
      }
    }
  }
  pub fn transfer_experiment(&mut self, transfer_configs: Vec<TransferConfig>, only_transfer: Option<bool>) {
    println!("Starting to evaluate {} transfer configurations", transfer_configs.len());
    create_dir_if_it_doesnt_exist(self.result_dir.as_ref());
    for transfer_config in transfer_configs {
      let start_from = transfer_config.0;
      let transfer_to = transfer_config.1;
      let env_name = start_from.env.get("name").expect("Name of Env needs to be in config for Transfer to work");
      create_dir_if_it_doesnt_exist(&format!("{}/{}", self.result_dir, env_name));
      let env_2_name = transfer_to.env.get("name").expect("Name of Env needs to be in config for Transfer to work");
      create_dir_if_it_doesnt_exist(&format!("{}/{}", self.result_dir, env_2_name));
      let algo_name = start_from.algo.get("name").expect("Name of Env needs to be in config for Transfer to work");
      let algo_dir = format!("{}/{}/{}", self.result_dir, env_name, algo_name);
      create_dir_if_it_doesnt_exist(algo_dir.as_str());
      let env_f: &mut EnvironmentMaker = self.environments.iter_mut().find(|env_to_check| {
        let env = (*env_to_check)(&start_from.env);
        &env.get_name() == env_name
      }).unwrap();
      let algo: &mut Box<dyn SelfTrainingAlgo> = {
        let mut to_ret = None;
        for algo in self.algorithms.iter_mut() {
          if &algo.get_name() == algo_name {
            to_ret = Some(algo);
          }
        }
        if let Some(algo) = to_ret {
          algo
        } else {
          &mut self.algorithms[0]
        }
      };
      if only_transfer.is_none() || only_transfer.unwrap() == false {
        println!("Running regular training");
        for current_run in 0..self.runs_per_env {
          RlExperimentHelper::run_experiment_for_config(env_f, algo, &start_from.env, Some(start_from.algo.clone()), current_run, None, &self.result_dir, &mut self.run_data, self.runs_per_env, self.steps_per_env, self.eval_episode_amount);
        }
      }
      // Make transfer experiment based on initial config 
      println!("Starting to transfer");
      for current_run in 0..self.runs_per_env {
        RlExperimentHelper::run_experiment_for_config(env_f, algo, &transfer_to.env, Some(transfer_to.algo.clone()), current_run, Some(start_from.env.clone()), &self.result_dir, &mut self.run_data, self.runs_per_env, self.steps_per_env, self.eval_episode_amount);
      }
    }
  }
  /*
    per env
     per algo
       multiple runs
         average + median
 */
  pub fn do_evaluation(&mut self) {
    println!("Starting to evaluate {} algorithms in {} environments. Training for {} steps and evaluating  {} episodes", self.algorithms.len(), self.environments.len(), self.steps_per_env,self.eval_episode_amount);
    create_dir_if_it_doesnt_exist(self.result_dir.as_ref());
    let envs = self.environments.iter_mut();
    for mut env_f in envs {
        let mut env = (*env_f)(&HashMap::new());
        let env_name = env.get_name();
        #[cfg(not(target_arch = "wasm32"))]
        {
          if metadata::<String>(format!("{}/{}", self.result_dir, env_name)).is_err() {
            create_dir::<String>(format!("{}/{}", self.result_dir, env_name)).unwrap();
          }
        }
        let algos = self.algorithms.iter_mut();
        // TODO: reset algo + save algo 
        for mut algo in algos {
          let algo_name = algo.get_name();
          #[cfg(not(target_arch = "wasm32"))]
          {
            let algo_dir = format!("{}/{}/{}", self.result_dir, env_name, algo_name);
            create_dir_if_it_doesnt_exist(algo_dir.as_str());
          }
          let env_config = env.get_config();
          for current_run in 0..self.runs_per_env {
            // Fill back in
            RlExperimentHelper::run_experiment_for_config(&mut env_f, &mut algo, &env_config, None, current_run, None, &self.result_dir, &mut self.run_data, self.runs_per_env, self.steps_per_env, self.eval_episode_amount);
          }
        }
    }
  }
}

// Env => {Algo => {id, episodeAmount, run_type, reuses, config}}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyzerEntry {
  pub id: String,
  pub episode_amount: usize,
  pub run_type: u8,
  pub is_eval_of: Option<String>,
  pub reuses: Option<String>,
  pub reused_by: Option<String>,
  pub env: String,
  pub env_config: HashMap<String, String>,
  pub algo: String,
  pub algo_config: Option<HashMap<String, String>>,
  pub load_from: String
}

impl AnalyzerEntry {
  pub fn new(run: &RunData, load_from: String) -> AnalyzerEntry {
    AnalyzerEntry {
      id: run.uid.clone(),
      episode_amount: run.episodes.len(),
      run_type: run.run_type,
      is_eval_of: run.is_eval_of.clone(),
      reuses: run.reuses.clone(),
      env_config: run.env_config.clone(),
      env: run.env.clone(),
      algo: run.algo.clone(),
      algo_config: run.hyperparams.clone(),
      reused_by: None,
      load_from
    }
  }
}

pub type AnalyzerIndex = Vec<AnalyzerEntry>;

pub fn enrich_episode(run: &mut RunData, episode: usize, env_init: &EnvironmentMaker) -> EnrichedEpisodeData {
  let mut env = env_init(&run.env_config);
  let episode_data = run.episodes.index(episode);
  if let Some(env_config) = &episode_data.env_params {
    env.load_config(env_config);
  }
  env.use_seed(episode_data.seed);
  env.reset();
  let mut rewards: RewardVector = Vec::with_capacity(episode_data.log.len());
  for log_entry in episode_data.log.iter() {
    let step = env.step(&log_entry);
    rewards.push(step.reward);
  }
  EnrichedEpisodeData {
    log: episode_data.log.clone(),
    seed: episode_data.seed, additional_info: None,
    rewards, algorithm_hyperparams: episode_data.algorithm_hyperparams.clone(),
    env_params: episode_data.env_params.clone(),
    task_completion: 0.0, episode_nr: episode
  }
}

pub fn enrich_episode_with_env<T: GymEnvironment>(env: &mut T, episode_data: EpisodeData) -> EnrichedEpisodeData {
  env.use_seed(episode_data.seed);
  env.reset();
  let mut rewards: RewardVector = Vec::with_capacity(episode_data.log.len());
  for log_entry in episode_data.log.iter() {
    let step = env.step(&log_entry);
    rewards.push(step.reward);
  }
  EnrichedEpisodeData {
    log: episode_data.log.clone(),
    seed: episode_data.seed, additional_info: None,
    rewards, env_params: episode_data.env_params.clone(),
    algorithm_hyperparams: episode_data.algorithm_hyperparams.clone(),
    task_completion: 0.0, episode_nr: 0
  }
}
pub fn enrich_episodes_with_env<T: GymEnvironment>(env: &mut T, episodes: &mut CompressedVec<EpisodeData>) -> Vec<EnrichedEpisodeData> {
  let mut enriched = Vec::with_capacity(episodes.len());
  for i in 0..episodes.len() {
    let episode_data = episodes.index(i);
    env.use_seed(episode_data.seed);
    env.reset();
    let mut rewards: RewardVector = Vec::with_capacity(episode_data.log.len());
    for log_entry in episode_data.log.iter() {
      let step = env.step(&log_entry);
      rewards.push(step.reward);
    }
    enriched.push(EnrichedEpisodeData {
      log: episode_data.log.clone(),
      seed: episode_data.seed, additional_info: None,
      rewards, env_params: episode_data.env_params.clone(),
      algorithm_hyperparams: episode_data.algorithm_hyperparams.clone(),
      task_completion: 0.0, episode_nr: i
    });
  }
  enriched
}

#[cfg(not(target_arch = "wasm32"))]
use std::fs::read;
use std::io::Write;
use serde_cbor::{from_slice, Value as CborValue};
use flate2::write::ZlibDecoder;
use std::collections::BTreeMap;
use ndarray::ArrayBase;
fn episodes_from_btree(episode_map: &BTreeMap<CborValue, CborValue>) -> CompressedVec<EpisodeData> {
  let mut max_len: u32 = 100;
  if let Some(ml_opt) = episode_map.get(&CborValue::Text("max_len_per_bucket".to_owned())) {
    match ml_opt {
      CborValue::Integer(ml_val) => {max_len = *ml_val as u32}
      _ => {}
    }
  }
  let mut new_episodes: CompressedVec<EpisodeData> = CompressedVec::with_max_len_per_bucket(max_len);

  if let Some(buckets_opt) = episode_map.get(&CborValue::Text("compressed_buckets".to_owned())) {
    match buckets_opt {
      CborValue::Array(buckets) => {
        for bucket in buckets {
          match bucket {
            CborValue::Bytes(bucket_bytes) => {
              let mut decompressor = ZlibDecoder::new(Vec::with_capacity(bucket_bytes.len()));
              decompressor.write_all(bucket_bytes).unwrap();
              let decompressed_file_contents = decompressor.finish().unwrap();
              let slice = decompressed_file_contents.as_slice();
              let episode_data: Vec<(u64, Vec<f64>)> = from_slice(slice).unwrap();
              for episode in episode_data {
                let actions = episode.1.iter().map(|v| ArrayBase::from(vec![*v]).into_dyn()).collect();
                new_episodes.push(EpisodeData::new(episode.0, actions));
              }
            }
            _ => {}
          }
        }
      }
      _ => {}
    }
  }
  new_episodes
}

#[allow(dead_code)]
fn rewards_from_vec(episode_map: &BTreeMap<CborValue, CborValue>) -> CompressedVec<EpisodeData> {
  let mut max_len: u32 = 100;
  if let Some(ml_opt) = episode_map.get(&CborValue::Text("max_len_per_bucket".to_owned())) {
    match ml_opt {
      CborValue::Integer(ml_val) => {max_len = *ml_val as u32}
      _ => {}
    }
  }
  let mut new_episodes: CompressedVec<EpisodeData> = CompressedVec::with_max_len_per_bucket(max_len);

  if let Some(buckets_opt) = episode_map.get(&CborValue::Text("compressed_buckets".to_owned())) {
    match buckets_opt {
      CborValue::Array(buckets) => {
        for bucket in buckets {
          match bucket {
            CborValue::Bytes(bucket_bytes) => {
              let mut decompressor = ZlibDecoder::new(Vec::with_capacity(bucket_bytes.len()));
              decompressor.write_all(bucket_bytes).unwrap();
              let decompressed_file_contents = decompressor.finish().unwrap();
              let slice = decompressed_file_contents.as_slice();
              let episode_data: Vec<(u64, Vec<f64>)> = from_slice(slice).unwrap();
              for episode in episode_data {
                let actions = episode.1.iter().map(|v| ArrayBase::from(vec![*v]).into_dyn()).collect();
                new_episodes.push(EpisodeData::new(episode.0, actions));
              }
            }
            _ => {}
          }
        }
      }
      _ => {}
    }
  }
  new_episodes
}

fn run_from_cbor_value(value: CborValue, file_name: &str) -> RunData  {
  let mut new_episodes = CompressedVec::new();
  let mut algo = "".to_owned();
  let mut env = "".to_owned();
  let mut uid = "".to_owned();
  let mut run_type = 1;
  let mut reuses = None;
  let mut is_eval_of = None;
  let hyperparams = None;
  let env_config = HashMap::new();
  let mut time_needed = 0;
  let mut reward_per_episode: Option<Vec<Reward>> = None;
  match value {
    CborValue::Map(map) => {
      if let Some(env_opt) = map.get(&CborValue::Text("env".to_owned())) {
        match env_opt {
          CborValue::Text(env_name) => {env = env_name.clone()}
          _ => {}
        }
      }
      if let Some(algo_opt) = map.get(&CborValue::Text("algo".to_owned())) {
        match algo_opt {
          CborValue::Text(algo_name) => {algo = algo_name.clone()}
          _ => {}
        }
      }
      if let Some(reuses_opt) = map.get(&CborValue::Text("reuses".to_owned())) {
        match reuses_opt {
          CborValue::Text(reuses_name) => {reuses = Some(reuses_name.clone())}
          _ => {}
        }
      }
      if let Some(is_eval_of_opt) = map.get(&CborValue::Text("is_eval_of".to_owned())) {
        match is_eval_of_opt {
          CborValue::Text(is_eval_of_name) => {is_eval_of = Some(is_eval_of_name.clone())}
          _ => {}
        }
      }
      if let Some(run_type_opt) = map.get(&CborValue::Text("run_type".to_owned())) {
        match run_type_opt {
          CborValue::Integer(run_type_name) => {run_type = *run_type_name as u8}
          _ => {}
        }
      }
      if let Some(time_needed_opt) = map.get(&CborValue::Text("time_needed".to_owned())) {
        match time_needed_opt {
          CborValue::Integer(time_needed_name) => {time_needed = *time_needed_name as u64}
          _ => {}
        }
      }
      if let Some(uid_opt) = map.get(&CborValue::Text("uid".to_owned())) {
        match uid_opt {
          CborValue::Text(uid_name) => {
            if uid_name.as_str() == "to_gen" {
              uid = file_name[file_name.rfind("/").unwrap()+1..file_name.rfind(".").unwrap()].to_string();
            } else {
              uid = uid_name.clone()
            }
          }
          _ => {}
        }
      }
      if let Some(episodes_opt) = map.get(&CborValue::Text("episodes".to_owned())) {
        match episodes_opt {
          CborValue::Map(compressed_vec) => {
            new_episodes = episodes_from_btree(compressed_vec);
          }
          _ => {}
        }
      }
      if let Some(rewards_opt) = map.get(&CborValue::Text("reward_per_episode".to_owned())) {
        match rewards_opt {
          CborValue::Array(reward_values) => {
            let mut rewards = Vec::with_capacity(reward_values.len());
            for reward_value in reward_values {
              match reward_value {
                CborValue::Float(val) => {
                  rewards.push(*val);
                }, _ => {}
              }
            }
            reward_per_episode = Some(rewards);
          }
          _ => {}
        }
      }
    },
    _ => {}
  }
  RunData {
    episodes: new_episodes,
    algo: algo,
    env: env,
    run_type: run_type,
    uid: uid,
    reuses: reuses,
    is_eval_of: is_eval_of,
    hyperparams: hyperparams,
    env_config: env_config,
    time_needed: time_needed,
    reward_per_episode
  }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_run_convert_python(path: &str) -> RunData {
  let compressed_file_contents = read(path).unwrap();
  let mut decompressor = ZlibDecoder::new(Vec::with_capacity(compressed_file_contents.len()));
  decompressor.write_all(&compressed_file_contents).unwrap();
  let decompressed_file_contents = decompressor.finish().unwrap();
  let slice = decompressed_file_contents.as_slice();
  let run_opt: Result<RunData, _> = from_slice(slice);
  if run_opt.is_err() {
    let py_run: CborValue = from_slice(slice).unwrap();
    return run_from_cbor_value(py_run, path);
  }
  run_opt.unwrap()
}


pub async fn async_load_run_convert_python(path: &str) -> RunData {
  let compressed_file_contents = xp_tools::read_file(path).await;
  let mut decompressor = ZlibDecoder::new(Vec::with_capacity(compressed_file_contents.len()));
  decompressor.write_all(&compressed_file_contents).unwrap();
  let decompressed_file_contents = decompressor.finish().unwrap();
  let slice = decompressed_file_contents.as_slice();
  let run_opt: Result<RunData, _> = from_slice(slice);
  if run_opt.is_err() {
    let py_run: CborValue = from_slice(slice).unwrap();
    return run_from_cbor_value(py_run, path);
  }
  run_opt.unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
use std::fs::{metadata, create_dir};
#[cfg(not(target_arch = "wasm32"))]
pub fn prepare_analyzer_data(result_dir: String, target_dir: String, evaluator: &mut RlExperimentHelper) {
  use walkdir::WalkDir;
  let mut analyzer_index: AnalyzerIndex = Vec::new();
  create_dir_if_it_doesnt_exist(target_dir.as_str());
  for entry_opt in WalkDir::new(result_dir) {
    if let Ok(entry) = entry_opt {
      let file_name = entry.path().to_str().unwrap();
      if file_name.ends_with(".tlr") {
        let mut run: RunData = load_run_convert_python(entry.path().to_str().unwrap());
        
        if metadata::<String>(format!("{}/{}", target_dir, run.env)).is_err() {
          create_dir::<String>(format!("{}/{}", target_dir, run.env)).unwrap();
        }
        if metadata::<String>(format!("{}/{}/{}", target_dir, run.env, run.algo)).is_err() {
          create_dir::<String>(format!("{}/{}/{}", target_dir, run.env, run.algo)).unwrap();
        }
        let env_init = evaluator.environments.iter_mut().find(|env_to_check| {
          let env = (*env_to_check)(&run.env_config);
          env.get_name() == run.env
        }).unwrap();
        let mut enriched_episodes: CompressedVec<EnrichedEpisodeData> = CompressedVec::new();
        for i in 0..run.episodes.len() {
          enriched_episodes.push(enrich_episode(&mut run, i, &env_init));
        }
        let enriched_run = RunDataEnriched::new(run.clone(), enriched_episodes.clone());
        let load_from = format!("{}/{}/{}.tlr", run.env, run.algo, run.uid);
        save_cbor_and_flate_to_path(format!("{}/{}x", target_dir, load_from).as_str(), &enriched_run);
        analyzer_index.push(AnalyzerEntry::new(&run, load_from));
      }
    }
  }
  save_json_to_path(format!("{}/index.json", target_dir), &analyzer_index);
}


#[cfg(not(target_arch = "wasm32"))]
pub fn create_run_index(result_dir: String) -> AnalyzerIndex {
  use walkdir::{WalkDir, DirEntry};
  // use rayon::prelude::*;
  println!("READING IN {}", result_dir);
  let mut analyzer_index: AnalyzerIndex = Vec::new();
  let walker = WalkDir::new(result_dir.clone());
  let mut entries: Vec<DirEntry> = Vec::new();
  for entry_opt in walker {
    if let Ok(entry) = entry_opt {
      entries.push(entry);
    }
  }
  for entry in entries {
    let file_name = entry.path().to_str().unwrap();
    if file_name.ends_with(".tlr") {
      println!("processing {}", file_name);
      let run: RunData = load_run_convert_python(entry.path().to_str().unwrap());
      let load_from = file_name.to_owned().chars().skip(result_dir.len()+1).take(file_name.len()).collect();
      println!("processing file {}", load_from);
      analyzer_index.push(AnalyzerEntry::new(&run, load_from));
    }
  }
  analyzer_index
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_all_runs(result_dir: &str, _page: usize) -> EvaluationData {
  use walkdir::{WalkDir, DirEntry};
  // use rayon::prelude::*;
  let mut runs: EvaluationData = Vec::new();
  let walker = WalkDir::new(result_dir);
  let mut entries: Vec<DirEntry> = Vec::new();
  for entry_opt in walker {
    if let Ok(entry) = entry_opt {
      entries.push(entry);
    }
  }
  for entry in entries {
    let file_name = entry.path().to_str().unwrap();
    if file_name.ends_with(".tlr") {
      println!("processing {}", file_name);
      let run: RunData = load_run_convert_python(entry.path().to_str().unwrap());
      runs.push(run);
    }
  }
  runs
}

#[cfg(target_arch = "wasm32")]
pub fn load_all_runs(result_dir: &str, page: usize) -> EvaluationData {
  futures::executor::block_on(async_load_cbor_and_flate_file(&format!("{}/page_{}.tlri", result_dir, page)))
}


use serde_json::Value;
use async_recursion::async_recursion;
#[async_recursion(?Send)]
pub async fn ipfs_load_runs_from_cid(ipfs_base_url: &str, ipfs_cid: &str) -> AnalyzerIndex {
  print!("Attempting IPFS stuff at {}  ", format!("{}/api/v0/file/ls?arg={}", ipfs_base_url, ipfs_cid));

  let mut analyzer_index: AnalyzerIndex = Vec::new();
  let mut run_list: Vec<(RunData, String)> = Vec::new();
  
  let raw_data = read_url_with_post(&format!("{}/api/v0/file/ls?arg={}", ipfs_base_url, ipfs_cid)).await;
  let parsed_data = String::from_utf8(raw_data).unwrap();
  let parsed_json: Value = serde_json::from_str(&parsed_data).unwrap();
  let file_url = xp_tools::ipfs_url_cat(ipfs_base_url, ipfs_cid);
  match parsed_json {
    Value::Object(obj) => {
      if let Some(sub_obj) = obj.get("Objects") {
        match sub_obj {
          Value::Object(objects_hashmap) => {
            if let Some(cid_obj) = objects_hashmap.get(ipfs_cid) {
              match cid_obj {
                Value::Object(cid_hashmap) => {
                  if let Some(type_obj) = cid_hashmap.get("Type") {
                    match type_obj {
                      Value::String(type_str) => {
                        match type_str.as_str() {
                          "File" => {
                            println!("We read {} as file", type_str);
                            let actual_file_data = read_url_with_post(&file_url).await;
                            let parsed_content: RunData = xp_tools::load_cbor_and_flate_from_vec(actual_file_data);
                            run_list.push((parsed_content, ipfs_cid.to_string()));
                          },
                          "Directory" => {
                            if let Some(links_obj) = cid_hashmap.get("Links") {
                              match links_obj {
                                Value::Array(link_arr) => {
                                  for sub_file in link_arr {
                                    match sub_file {
                                      Value::Object(sub_file_obj) => {
                                        if let Some(sub_file_type) = sub_file_obj.get("Type") {
                                          if sub_file_type == "File" {
                                            if let Some(sub_file_name) = sub_file_obj.get("Name") {
                                              match sub_file_name {
                                                Value::String(sub_file_name_str) => {
                                                  if sub_file_name_str.ends_with(".tlr") {
                                                    if let Some(sub_file_cid) = sub_file_obj.get("Hash") {
                                                      match sub_file_cid {
                                                        Value::String(sub_file_cid_str) => {
                                                          let sub_runs = ipfs_load_runs_from_cid(ipfs_base_url, sub_file_cid_str).await;
                                                          for first_sub_run in sub_runs {
                                                            analyzer_index.push(first_sub_run);
                                                          }
                                                        },
                                                        _ => {}
                                                      }
                                                    }
                                                  }
                                                },
                                                _ => {}
                                              }
                                            }
                                          }
                                        }
                                      }
                                      _ => {}
                                    }
                                  }
                                }
                                _ => {}
                              }
                            }
                          },
                          _ => {}
                        }
                      },
                      _ => {}
                    }
                  }
                },
                _ => {}
              }
            }
          }
          _ => {}
        }
      }
    }
    _ => {}
  }

  for (run, run_ipfs_cid) in run_list {
    let load_from = format!("{}", run_ipfs_cid);
    println!("processing file {}", load_from);
    analyzer_index.push(AnalyzerEntry::new(&run, load_from));
  }
  analyzer_index
}

use regex::Regex;

pub async fn ipfs_gateway_run_from_cid(ipfs_base_url: &str, ipfs_cid: &str) -> RunData {
  
  let load_from = format!("{}/ipfs/{}", ipfs_base_url, ipfs_cid);
  print!("Fetching run at {}  ", load_from);
  let raw_data = read_url(&load_from).await;
  let run: RunData = xp_tools::load_cbor_and_flate_from_vec(raw_data);
  run
}

pub async fn ipfs_gateway_runs_from_cid(ipfs_base_url: &str, ipfs_cid: &str) -> AnalyzerIndex {
  print!("Attempting IPFS stuff at {}  ", format!("{}/api/v0/file/ls?arg={}", ipfs_base_url, ipfs_cid));
  let mut analyzer_index: AnalyzerIndex = Vec::new();
  
  let raw_data = read_url(&format!("{}/ipfs/{}", ipfs_base_url, ipfs_cid)).await;
  let parsed_data = String::from_utf8(raw_data).unwrap();
  let href_regex = Regex::new("href=\"(.*)?\"").unwrap(); // should be wrapped in lazy_static
  let replay_cids: Vec<String> = href_regex.captures_iter(&parsed_data).filter_map(|cap| {
    if let Some(link) = cap.get(1) {
      let linkstr = link.as_str();
      if linkstr.ends_with(".tlr") && linkstr.contains("?filename") {
        let cid: &str = &linkstr[6..linkstr.find("?").unwrap()];
        return Some(cid.to_owned());
      }
    }
    None
  }).collect();
  for cid in replay_cids {
    let run: RunData = ipfs_gateway_run_from_cid(ipfs_base_url, &cid).await;
    analyzer_index.push(AnalyzerEntry::new(&run, format!("{}/ipfs/{}", ipfs_base_url, cid)));
  }
  println!("GOT INDEX {:?}", analyzer_index.len());
  analyzer_index
}


pub fn get_ipfs_url() -> String {
  if xp_tools::get_env_variable("CI").unwrap_or("false".to_string()).to_lowercase() == "true" {
    "http://dweb.link".to_owned()
  } else {
    "http://127.0.0.1:8080".to_owned()
  }
}

#[cfg(test)]
pub mod test {
  use super::{RlExperimentHelper, prepare_analyzer_data, enrich_episode, EnrichedEpisodeData, avg, EnvironmentMaker};
  use std::fs::metadata;
  use compressed_vec::CompressedVec;
  use rand::{Rng, RngCore};
  use xp_tools::rng::from_seed;
  use crate::{Action, GymEnvironment,RunData, Observation, RlAlgorithm, SelfTrainingAlgo, Space, Step, TransferConfigDetail, env::zero_or_one::EnvZeroOrOne, space_to_num};
  use ndarray::ArrayBase;
  use std::collections::HashMap;
  
  pub struct RandomAlgorithm {action_space: Space}
  
  impl RandomAlgorithm {
    pub fn new(action_space: Space) -> RandomAlgorithm {
      RandomAlgorithm {action_space}
    }
  }
  
  impl RlAlgorithm for RandomAlgorithm {
    fn set_observation_shape(&mut self, _: Space) {}
    fn get_name(&self) -> String {"random".to_owned()}
    fn save(&mut self, save_path: &str) {
      std::fs::write(save_path, "randomalgo").unwrap();
    }
    fn load(&mut self, _load_path: &str) {
    }
    fn act(&mut self, _obs: Observation) -> Action {
      let amount_of_actions = space_to_num(&self.action_space);
      let (mut rng, _) = from_seed(None);
      let action = rng.gen_range(0..amount_of_actions);
      ArrayBase::from(vec![action as f64]).into_dyn()
    }
    fn get_hyperparams(&mut self) -> HashMap<String, String> {
      HashMap::new() // TODO
    }
    fn load_hyperparams(&mut self, _config: &HashMap<String, String>){
      // TODO
    }
    fn reset(&mut self) {}
  }
  impl SelfTrainingAlgo for RandomAlgorithm {
    fn train_on_env(&mut self, mut rust_env: Box<dyn ReplayableGymEnvironment>, seed: Option<u64>, number_of_steps: usize) {
      let mut current_step = 0;
      let (mut rng, mut env_seed) = from_seed(seed);
      rust_env.use_seed(env_seed);
      let mut obs = rust_env.reset();
      let mut b = Vec::new();
      while current_step < number_of_steps {
        current_step += 1;
        b.clear();
        b.push(obs.clone());
        let arr = ArrayBase::from(vec![rng.gen_range(0..5) as f64]).into_dyn();
        let step_result: Step = rust_env.step(&arr);
        if step_result.is_done {
          env_seed = rng.next_u64();
          rust_env.use_seed(env_seed);
          obs = rust_env.reset();
        } else {
          obs = step_result.obs.clone();
        }
      }
    }
  }
  


  pub fn test_file_exists(path: &str) {
    assert!(metadata(path).is_ok());
  }

  #[test]
  fn test_regular_experiment_runner<'a, 'b>() {
    let env = EnvZeroOrOne::default();
    let algorithms: Vec<Box<dyn SelfTrainingAlgo>> = vec![Box::new(RandomAlgorithm::new(env.action_space()))];
    let environments: Vec<EnvironmentMaker> = vec![Box::new(|_| {
      Box::new(EnvZeroOrOne::default())
    })];
    let e2: Vec<EnvironmentMaker> = vec![Box::new(|_| {
      Box::new(EnvZeroOrOne::default())
    })];
    let result_dir = "/tmp/tlf_test_eval";
    let analysis_dir = "/tmp/tlf_test_eval_analysis";
    let mut evaluator = RlExperimentHelper::with_config(result_dir.to_owned(), algorithms, environments);
    evaluator.do_evaluation();
    test_file_exists(format!("{}/run_data.bin", result_dir).as_str());
    for run in evaluator.run_data.iter() {
      test_file_exists(format!("{}/{}/{}/{}_weights", result_dir, run.env, run.algo, run.uid).as_str());
      test_file_exists(format!("{}/{}/{}/{}.tlr", result_dir, run.env, run.algo, run.uid).as_str());
    }
    let mut run = evaluator.run_data[0].clone();
    let mut env = EnvZeroOrOne::default();
    let mut enriched_episodes: CompressedVec<EnrichedEpisodeData> = CompressedVec::new();
    for i in 0..run.episodes.len() {
      enriched_episodes.push(enrich_episode(&mut run, i, &e2[0]));
    }
    
    for idx in 0..enriched_episodes.len() {
      let enriched_episode = enriched_episodes.index(idx);
      let achieved_rewards = enriched_episode.rewards.clone();
      let replay = enriched_episode.log.clone();
      env.use_seed(enriched_episode.seed);
      let mut rewards = Vec::with_capacity(achieved_rewards.len());
      for entry in replay {
        let step: Step = env.step(&entry);
        rewards.push(step.reward);
      }
      for index in 0..rewards.len() {
        assert_eq!(rewards[index], achieved_rewards[index]);
      }
      assert_eq!(avg(&achieved_rewards), avg(&rewards));
    }
    prepare_analyzer_data(result_dir.to_owned(), analysis_dir.to_owned(), &mut evaluator);
    test_file_exists(format!("{}/index.json", analysis_dir).as_str());
    for run in evaluator.run_data.iter() {
      test_file_exists(format!("{}/{}/{}/{}.tlrx", analysis_dir, run.env, run.algo, run.uid).as_str());
    }
  }
  
  #[test]
  fn test_trl_experiment_runner<'a, 'b>() {
    let env = EnvZeroOrOne::default();
    let algorithms: Vec<Box<dyn SelfTrainingAlgo>> = vec![Box::new(RandomAlgorithm::new(env.action_space()))];
    let environments: Vec<EnvironmentMaker> = vec![Box::new(|_| {
      Box::new(EnvZeroOrOne::default())
    })];
    let result_dir = "/tmp/trl_test_eval";
    let mut evaluator = RlExperimentHelper::with_config(result_dir.to_owned(), algorithms, environments);
    let mut env_config_1 = HashMap::new();
    env_config_1.insert("name".to_string(), "zero_or_one".to_string());
    let mut algo_config_1 = HashMap::new();
    algo_config_1.insert("name".to_string(), "random".to_string());
    evaluator.transfer_experiment(vec![
      (TransferConfigDetail::new(env_config_1.clone(), algo_config_1.clone()), TransferConfigDetail::new(env_config_1, algo_config_1))
    ], Some(false));
    test_file_exists(format!("{}/run_data.bin", result_dir).as_str());
    for run in evaluator.run_data.iter() {
      test_file_exists(format!("{}/{}/{}/{}_weights", result_dir, run.env, run.algo, run.uid).as_str());
      test_file_exists(format!("{}/{}/{}/{}.tlr", result_dir, run.env, run.algo, run.uid).as_str());
    }
    let transferred_runs = evaluator.run_data.iter().filter(|run| run.reuses.is_some()).collect::<Vec<&RunData>>().len();
    assert!(transferred_runs > 0);
  }
  // use super::{get_ipfs_url, ipfs_gateway_runs_from_cid, ipfs_load_runs_from_cid};
  #[tokio::test]
  async fn test_ipfs_load<'a, 'b>() {
    if xp_tools::get_env_variable("CI").unwrap_or("false".to_string()).to_lowercase() != "true" {
      // let full_directory = ipfs_load_runs_from_cid(&get_ipfs_url(), "bafybeicuodgfdvmcpzl7o3r33imcijq4lryfxgqzap2dysygf6j7m4slyu").await;
      // assert!(full_directory.len() == 1);
    }
    // let single_file = ipfs_load_runs_from_cid(get_ipfs_url(), "QmXmPANjMnEXNf61wCfAuURmFCF9C97Vu653M45wTZxtaa").await;
    // assert!(single_file.len() == 1);
  }
  #[tokio::test]
  async fn test_ipfs_load_via_html<'a, 'b>() {
    if xp_tools::get_env_variable("CI").unwrap_or("false".to_string()).to_lowercase() != "true" {
      // let full_directory = ipfs_load_runs_from_cid(&get_ipfs_url(), "bafybeicuodgfdvmcpzl7o3r33imcijq4lryfxgqzap2dysygf6j7m4slyu").await;
      // assert!(full_directory.len() == 1);
    }
    // let single_file = ipfs_gateway_runs_from_cid(&get_ipfs_url(), "bafybeibozgxnflyw2ok2lubc5nonfu4klg7izzktowkczuidu3vtzf57b4").await;
    // assert!(single_file.len() > 0);
  }
}