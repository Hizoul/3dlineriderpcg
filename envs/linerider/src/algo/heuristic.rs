use std::ops::Sub;
use std::collections::HashMap;
use bevy::prelude::Vec3;
use ndarray::ArrayBase;
use rusty_gym::{Observation, Action};

use crate::util::consts::{ACTION_TYPE_FREE_POINTS_RELATIVE, ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE, TARGET_ABOVE_START, TARGET_SAME_HEIGHT_AS_START, TP_ACCELERATE, TP_NORMAL};

pub const EMPTY_POINT_IN_OBSERVATION: [f64; 4] = [-1.0, -1.0, -1.0, -1.0];


/**
 * Heuristic made for Target Type A to B with Action Space Relative
 */
pub fn straight_line_heuristica(observation: &Observation, config: &HashMap<String, String>, multiplier: Option<f32>) -> Action {
  let obs = observation.as_slice().expect("");
  let obs_len = obs.len();
  let mut current_pos = Vec3::ZERO;
  let mut found_pos = false;
  let mut current_index = 0;
  'LAST_POS_SEARCH: while !found_pos {
    if current_index+3 >= obs_len {
      break 'LAST_POS_SEARCH;
    }
    let current_point = &obs[current_index..current_index+4];
    if EMPTY_POINT_IN_OBSERVATION.eq(current_point) {
      found_pos = true;
      let previous_point = &obs[current_index-4..current_index];
      current_pos = Vec3::new(previous_point[0] as f32, previous_point[1] as f32,previous_point[2] as f32);
    }
    current_index += 4;
  }
  let goal_pos_arr = &obs[obs_len-4..obs_len];
  let goal_pos = Vec3::new(goal_pos_arr[0] as f32,goal_pos_arr[1] as f32, goal_pos_arr[2] as f32);
  let mut direction = goal_pos.sub(current_pos).normalize();
  if let Some(dist_multiplier) = multiplier {
    direction *= dist_multiplier;
  }
  let action_type = config.get("action_type").expect("Action Type must be in Config");
  let action_type_val = action_type.parse::<u8>().expect("Action Type is of u8 type");
  match action_type_val {
    ACTION_TYPE_FREE_POINTS_RELATIVE => {
      ArrayBase::from(vec![direction.x as f64, direction.y as f64, direction.z as f64]).into_dyn()
    },
    ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE => {
      let target_type = config.get("target_type").expect("Target Type must be in Config");
      let target_type_val = target_type.parse::<u8>().expect("Target Type is of u8 type");
      let track_type = match target_type_val {
        TARGET_ABOVE_START | TARGET_SAME_HEIGHT_AS_START => {TP_ACCELERATE},
        _ => {TP_NORMAL}
      };
      ArrayBase::from(vec![direction.x as f64, direction.y as f64, direction.z as f64, track_type as f64]).into_dyn()
    }
    _ => {panic!("Heuristic can't handle action_type {}", action_type_val)}
  }
}

/**
 * Heuristic that includes checkopint
 */
pub fn straight_line_heuristic_general(observation: &Observation, config: &HashMap<String, String>, multiplier: Option<f32>) -> Action {
  let obs = observation.as_slice().expect("");
  let obs_len = obs.len();
  let mut current_pos = Vec3::ZERO;
  let mut goal_pos = Vec3::ZERO;
  let mut found_pos = false;
  let mut current_index = 0;
  'LAST_POS_SEARCH: while !found_pos {
    if current_index+3 >= obs_len {
      break 'LAST_POS_SEARCH;
    }
    let current_point = &obs[current_index..current_index+4];
    if EMPTY_POINT_IN_OBSERVATION.eq(current_point) {
      found_pos = true;
      let previous_point = &obs[current_index-4..current_index];
      current_pos = Vec3::new(previous_point[0] as f32, previous_point[1] as f32,previous_point[2] as f32);
    }
    current_index += 4;
  }
  let mut found_pos = false;
  'GOAL_POS_SEARCH: while !found_pos {
    if current_index+3 >= obs_len {
      break 'GOAL_POS_SEARCH;
    }
    let current_point = &obs[current_index..current_index+4];
    if current_point[3] == 4.0 {
      let potential_goal_pos = Vec3::new(current_point[0] as f32, current_point[1] as f32,current_point[2] as f32);
      if potential_goal_pos.distance(current_pos) > 0.9 {
        found_pos = true;
        goal_pos = potential_goal_pos;
        goal_pos.y -= 0.25;
        goal_pos.x -= 0.25;
      }
    }
    current_index += 4;
  }
  let action_type = config.get("action_type").expect("Action Type must be in Config");
  let action_type_val = action_type.parse::<u8>().expect("Action Type is of u8 type");
  if found_pos == false {
    match action_type_val {
      ACTION_TYPE_FREE_POINTS_RELATIVE => {
        ArrayBase::from(vec![0.0, 0.0, 0.0]).into_dyn()
      },
      ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE => {
        ArrayBase::from(vec![0.0, 0.0, 0.0, 0.0]).into_dyn()
      }
      _ => {panic!("Heuristic can't handle action_type {}", action_type_val)}
    }
  } else {

    let mut direction = goal_pos.sub(current_pos).normalize();
    if let Some(dist_multiplier) = multiplier {
      direction *= dist_multiplier;
    }
    match action_type_val {
      ACTION_TYPE_FREE_POINTS_RELATIVE => {
        ArrayBase::from(vec![direction.x as f64, direction.y as f64, direction.z as f64]).into_dyn()
      },
      ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE => {
        let track_type = if current_pos.y < goal_pos.y {TP_ACCELERATE} else {TP_NORMAL};
        ArrayBase::from(vec![direction.x as f64, direction.y as f64, direction.z as f64, track_type as f64]).into_dyn()
      }
      _ => {panic!("Heuristic can't handle action_type {}", action_type_val)}
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{env::LineRider3DEnv, simulator::LineRiderSim, util::consts::*, algo::heuristic::straight_line_heuristica};
  use rusty_gym::{GymEnvironment, ReplayableGymEnvironment, GymRecorder};
  use bevy::prelude::Vec3;
  #[test]
  fn straight_line_heuristic() {
    let mut sim: LineRiderSim = LineRiderSim::default();
    sim.config.reward_type = vec![
      REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL, REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END,
      REWARD_TRACK_CLOSER_TO_GOAL_IN_STEP, REWARD_GOAL_REACHED_BY_BOTH_ONLY,
      REWARD_GOAL_REACHED_BY_TRACK,  REWARD_SCOLD_INVALID_ACTION,
      REWARD_DISTANCE_TO_GOAL_IN_SIMULATION_IF_TRACK_REACHED_GOAL]; 
    sim.config.target_type = TARGET_RANDOM_START_AND_END;
    sim.config.action_type = ACTION_TYPE_FREE_POINTS_RELATIVE;
    sim.config.step_limit = 50;
    sim.config.skip_collision_check_on_last_x_pieces = 999;
    sim.config.max_piece_length = 2.0;
    sim.set_goal_position(Vec3::new(0.0, 0.0, 4.0));
    sim.set_max_width(50.0);
    let mut env: LineRider3DEnv = LineRider3DEnv::new(sim, None);
    // let name = env.get_name();
    let env_config = env.get_config();
    env.load_config(&env_config);
    let mut rec = GymRecorder::new(Box::new(env), None);
    // env.use_seed(42);
    for _ in 0..2 {
      let mut obs = rec.reset();
      let mut done = false;
      // let mut last_reward = -1.0;
      while !done {
        let config = rec.get_config();
        let action = &straight_line_heuristica(&obs, &config, None);
        let res = rec.step(action);
        done = res.is_done;
        // last_reward = res.reward;
        obs = res.obs;
      }
      // assert!(last_reward > 1.0);
    }
    let episode_data = rec.data.clone();
    let mut episodes = {
      let unlocked_eps = episode_data.lock().unwrap();
      unlocked_eps.clone()
    };
    episodes.finalize();
    // let mut new_run_data = RunData::new(RUNTYPE_TRAINING, name, "Heuristic".to_owned(), episodes, env_config.clone(), None, 0, Some("heuristic_solution".to_owned()), None);
    // save_cbor_and_flate_to_path("heuristic_solution.tlr", &new_run_data);
  }
}
