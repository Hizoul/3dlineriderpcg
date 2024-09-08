use std::collections::HashMap;
use rusty_gym::{GymEnvironment, GymRecorder, Observation, ReplayableGymEnvironment};
use linerider::{
  algo::heuristic::straight_line_heuristic_general,
  env::LineRider3DEnv, simulator::LineRiderSim,
  util::consts::*
};
use rayon::prelude::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn make_heuristic_logs() {
  // TODO: runid is not correct
  let m = MultiProgress::new();
  m.println("Starting to generate Heuristic Logs").unwrap();
  let size = 10;
  let target_types = vec![TARGET_RANDOM_START_AND_END, TARGET_SAME_HEIGHT_AS_START, TARGET_ABOVE_START];//TARGET_RANDOM_START_AND_END, TARGET_SAME_HEIGHT_AS_START, TARGET_ABOVE_START];
  target_types.par_iter().for_each(|target_type| {
    let sim = LineRiderSim::new(false);
    let mut env = LineRider3DEnv::new(sim, None);
    let mut config = env.get_config();
    config.insert("action_type".to_owned(), ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE.to_string());
    config.insert("reward_type".to_owned(), "[254]".to_owned());
    config.insert("target_type".to_owned(), target_type.to_string());
    config.insert("booster_strength".to_owned(), "0.26".to_string());
    config.insert("step_limit".to_owned(), size.to_string());
    config.insert("max_width".to_owned(), size.to_string());
    config.insert("simulation_steps".to_owned(), ((1000/80)*(80*size)).to_string());
    make_episodes_for_heuristic(10000, &config, &m, None);
  });
}
pub fn make_heuristic_logs_size() {
  // TODO: runid is not correct
  let m = MultiProgress::new();
  m.println("Starting to generate Heuristic Logs").unwrap();
  let sizes = vec![10, 20, 30, 40, 50];
  sizes.par_iter().for_each(|size| {
    let sim = LineRiderSim::new(false);
    let mut env = LineRider3DEnv::new(sim, None);
    let mut config = env.get_config();
    config.insert("action_type".to_owned(), ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE.to_string());
    config.insert("reward_type".to_owned(), "[254]".to_owned());
    config.insert("target_type".to_owned(), TARGET_RANDOM_START_AND_END.to_string());
    config.insert("step_limit".to_owned(), size.to_string());
    config.insert("max_width".to_owned(), size.to_string());
    config.insert("simulation_steps".to_owned(), ((1000/80)*(80*size)).to_string());
    make_episodes_for_heuristic(10000, &config, &m, Some(&format!("size_{}", size)));
  });
}

pub fn make_episodes_for_heuristic(episode_amount: usize, config: &HashMap<String, String>, m: &MultiProgress, name_addon_opt: Option<&str>) {
  let target_type = config.get("target_type").expect("Target Type must be in Config");
  let target_type_val = target_type.parse::<u8>().expect("Target Type is of u8 type");
  let name_addon = if name_addon_opt.is_some() {name_addon_opt.unwrap()} else {
    match target_type_val {
      TARGET_ABOVE_START => {"up"},
      TARGET_SAME_HEIGHT_AS_START => {"same"},
      TARGET_RANDOM_WITH_CHECKPOINT_ABOVE => {"chk_up"},
      TARGET_RANDOM_WITH_CHECKPOINT_BELOW => {"chk_down"},
      _ => {"down"}
    }
  };
  let name_to_use = format!("heuristic_{}", name_addon);
  let sty = ProgressStyle::with_template(
    "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
  )
  .unwrap()
  .progress_chars("##-");
  let pb = m.add(ProgressBar::new(episode_amount as u64));
  pb.set_style(sty.clone());
  let sim = LineRiderSim::new(false);
  let mut env = LineRider3DEnv::new(sim, None);
  env.load_config(config);
  let mut recording_env = GymRecorder::new(Box::new(env), Some(name_addon.to_owned()));

  let mut obs: Observation = recording_env.reset();
  let mut current_episode = 0;
  while current_episode < episode_amount {
    let step = recording_env.step(&straight_line_heuristic_general(&obs, config, None));
    if step.is_done {
      obs = recording_env.reset();
      current_episode += 1;
      pb.inc(1);
    } else {
      obs = step.obs;
    }
  }
  recording_env.finalize("heuristic", "");
}

