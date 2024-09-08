use std::collections::HashMap;
use rusty_gym::{GymEnvironment, GymRecorder, Observation, ReplayableGymEnvironment};
use linerider::{
  algo::heuristic::{straight_line_heuristic_general},
  env::LineRider3DEnv, simulator::LineRiderSim,
  util::consts::{ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE,
    TARGET_ABOVE_START, TARGET_SAME_HEIGHT_AS_START,
    TARGET_RANDOM_START_AND_END
  }
};
use rayon::prelude::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn find_optimal_booster_strength() {
  let m = MultiProgress::new();
  m.println("Starting to test Booster Strength Success").unwrap();
  let mut booster_strengths: Vec<f32> = vec![];
  for i in 1..9 {
    booster_strengths.push(i as f32 * 0.025);
  }
  let mut results: Vec<(f32, usize)> = booster_strengths.par_iter().map(|booster_strength| {
    let sim = LineRiderSim::new(false);
    let mut env = LineRider3DEnv::new(sim, None);
    let mut config = env.get_config();
    config.insert("action_type".to_owned(), ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE.to_string());
    config.insert("reward_type".to_owned(), "[254, 0]".to_owned());
    config.insert("target_type".to_owned(), TARGET_ABOVE_START.to_string());
    config.insert("booster_strength".to_owned(), booster_strength.to_string());
    config.insert("max_up_angle".to_owned(), "[-90.0, 90.0]".to_string());
    config.insert("simulation_steps".to_owned(), ((1000/80)*600).to_string());
    let success = make_episodes_for_heuristic(100000, &config, &m, &format!("boost_{}", booster_strength));
    (*booster_strength, success)
  }).collect();
  results.sort_by(|a, b| a.0.total_cmp(&b.0));
  println!("RESULTS ARE {:?}", results);
}

pub fn find_starting_force() {
// pub fn find_optimal_booster_strength_() {
  let m = MultiProgress::new();
  m.println("Starting to test Booster Strength Success").unwrap();
  let mut starting_forces: Vec<f32> = vec![0.01, 0.1, 0.25];
  let mut results: Vec<(f32, usize)> = starting_forces.par_iter().map(|starting_force_multiplier| {
    let sim = LineRiderSim::new(false);
    let mut env = LineRider3DEnv::new(sim, None);
    let mut config = env.get_config();
    config.insert("action_type".to_owned(), ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE.to_string());
    config.insert("reward_type".to_owned(), "[254, 0]".to_owned());
    config.insert("smooth_free_points".to_owned(), "true".to_owned());
    config.insert("target_type".to_owned(), TARGET_RANDOM_START_AND_END.to_string());
    config.insert("simulation_steps".to_owned(), ((1000/80)*600).to_string());
    config.insert("starting_force_multiplier".to_owned(), starting_force_multiplier.to_string());
    let success = make_episodes_for_heuristic(100000, &config, &m, &format!("startingforce_{}", starting_force_multiplier));
    (*starting_force_multiplier, success)
  }).collect();
  results.sort_by(|a, b| a.0.total_cmp(&b.0));
  println!("RESULTS ARE {:?}", results);
}

// more bad 80, 360 "[20.0, 60.0]", "[30.0, 70.0]", "[40.0, 80.0]", "[50.0, 100.0]", "[120.0, 360.0]", "[100.0, 360.0]", "[140.0, 360.0]", "[160.0, 360.0]", "[180.0, 360.0]"
// bad: "[40.0, 100.0]", "[40.0, 150.0]", 50 360; 
// pretty good: 40 350; 40 200;

pub fn find_optimal_booster_strength__() {
  let m = MultiProgress::new();
  m.println("Starting to test Booster Strength Success").unwrap();
  // let mut booster_strengths: Vec<&str> = vec!["[40.0, 200.0]", "[60.0, 200.0]", "[80.0, 200.0]", "[100.0, 200.0]", "[120.0, 200.0]", "[140.0, 200.0]", "[150.0, 200.0]"];
  let mut booster_strengths: Vec<&str> = vec!["[-90.0, 90.0]"];
  
  // let mut booster_strengths: Vec<&str> = vec!["[40.0, 80.0]", "[40.0, 100.0]", "[40.0, 120.0]", "[60.0, 100.0]", "[80.0, 100.0]", "[80.0, 120.0]", ];
  // let mut booster_strengths: Vec<&str> = vec!["[40.0, 120.0]", "[60.0, 120.0]", "[80.0, 120.0]", "[100.0, 120.0]", "[120.0, 120.0]", "[140.0, 120.0]", "[150.0, 120.0]"];
  
  let mut results: Vec<(String, usize)> = booster_strengths.iter().map(|booster_strength| {
    let sim = LineRiderSim::new(false);
    let mut env = LineRider3DEnv::new(sim, None);
    let mut config = env.get_config();
    config.insert("action_type".to_owned(), ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE.to_string());
    config.insert("reward_type".to_owned(), "[254, 0, 1]".to_owned());
    config.insert("target_type".to_owned(), TARGET_ABOVE_START.to_string());
    config.insert("booster_strength".to_owned(), "0.3".to_owned());
    config.insert("max_up_angle".to_owned(), booster_strength.to_string());
    config.insert("simulation_steps".to_owned(), ((1000/80)*600).to_string());
    let success = make_episodes_for_heuristic(1000, &config, &m, &format!("boost_{}", booster_strength));
    (booster_strength.to_string(), success)
  }).collect();
  // results.sort_by(|a, b| a.0.total_cmp(&b.0));
  println!("RESULTS ARE {:?}", results);
}

pub fn make_episodes_for_heuristic(episode_amount: usize, config: &HashMap<String, String>, m: &MultiProgress, name_addon: &str) -> usize {
  let target_type = config.get("target_type").expect("Target Type must be in Config");
  let target_type_val = target_type.parse::<u8>().expect("Target Type is of u8 type");
  let name_to_use = name_addon.to_string();
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
  let mut success_counter = 0;
  while current_episode < episode_amount {
    let step = recording_env.step(&straight_line_heuristic_general(&obs, config, Some(1.5)));
    if step.is_done {
      if step.reward >= 2.0 {
        success_counter += 1;
      }
      obs = recording_env.reset();
      current_episode += 1;
      pb.inc(1);
    } else {
      obs = step.obs;
    }
  }
  m.println(format!("For strength {} success is {} / {} ({}%)", name_addon, success_counter, episode_amount, (success_counter as f32 / episode_amount as f32) * 100.0)).unwrap();
  recording_env.finalize("uptest", "");
  success_counter
}

