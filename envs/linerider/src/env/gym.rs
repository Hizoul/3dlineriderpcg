
use bevy::math::{Vec3, Quat};
use rusty_gym::{ReplayableGymEnvironment,
  GymEnvironment, Step, Observation, Action, Space,
  util::rng::get_rng_for_type
};

use crate::{util::consts::*, algo::heuristic::straight_line_heuristica};

use super::LineRider3DEnv;

impl GymEnvironment for LineRider3DEnv {
  fn use_seed(&mut self, seed: u64) {
    let (new_rng, new_seed) = get_rng_for_type(&None, Some(seed));
    self.rng = new_rng;
    self.used_seed = new_seed;
  }
  fn reset(&mut self) -> Observation {
    self.reset_state();
    self.make_obs()
  }

  fn step(&mut self, action: &Action) -> Step {
    let mut is_done = false;
    let mut reward = 0.0;
    let obs = self.make_obs();
    if self.current_step >= self.sim.config.step_limit {
      is_done = true;
      reward += self.get_reward_through_simulation().0;
    } else {
      let added = match self.sim.config.action_type {
        ACTION_TYPE_FREE_POINTS_RELATIVE | ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE => {
          let track_type = if self.sim.config.action_type == ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE {action[3] as u8} else {TP_NORMAL};
          if self.sim.config.reward_type.contains(&REWARD_USING_BOOSTER_TYPE_TRACK) {
            if track_type == TP_ACCELERATE {
              reward += 0.01;
            }
          }
          let last_point = self.lines[self.lines.len()-1].0;
          let new_point = (Vec3::new(last_point.x + action[0] as f32, last_point.y + action[1] as f32, last_point.z + action[2] as f32), track_type);
          self.add_line_for_point(new_point)
        },
        ACTION_TYPE_FREE_POINTS | ACTION_TYPE_FREE_POINTS_WITH_TP => {
          let track_type = if self.sim.config.action_type == ACTION_TYPE_FREE_POINTS_WITH_TP {action[3] as u8} else {TP_NORMAL};
          let new_point = (Vec3::new(action[0] as f32, action[1] as f32, action[2] as f32), track_type);
          self.add_line_for_point(new_point)
        },
        ACTION_TYPE_RADIAL | ACTION_TYPE_RADIAL_WITH_TP => {
          let track_type = if self.sim.config.action_type == ACTION_TYPE_RADIAL_WITH_TP {action[3] as u8} else {TP_NORMAL};
          let yaw = Quat::from_rotation_y(action[1] as f32);
          let pitch = Quat::from_rotation_x(action[2] as f32);
          let distance = action[0] as f32;
          let last_point = self.lines[self.lines.len()-1].0;
          let prev_last_point = if self.lines.len() > 1 {self.lines[self.lines.len()-2].0} else {Vec3::Z};
          let direction = (last_point - prev_last_point).normalize();
          let rotation = yaw * pitch;
          let moved_point = rotation * (direction * distance);
          let new_point = (last_point + moved_point, track_type);
          self.add_line_for_point(new_point)
        },
        _ => {
          self.add_line_for_action(action[0] as i64)
        }
      };
      if added {
        if self.sim.config.reward_type.contains(&REWARD_VALID_ACTION_CHOSEN) {
          reward += 0.0001;
        }
        if self.sim.config.reward_type.contains(&REWARD_TRACK_CLOSER_TO_GOAL_IN_STEP) {
          let dist = self.sim.config.goal_pos.distance(self.lines[self.lines.len()-1].0);
          let distance_difference = self.track_distance_of_last_step - dist;
          if distance_difference > 0.0 {
            reward += 0.01;
          }
        }
        if (self.sim.config.action_type == ACTION_TYPE_FREE_POINTS_RELATIVE || self.sim.config.action_type == ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE) && self.sim.config.reward_type.contains(&REWARD_MIMIC_STRAIGHT_LINE_HEURISTIC) {
          let optimal_action = straight_line_heuristica(&obs, &self.get_config(), None);
          let optimal_vec = Vec3::new(optimal_action[0] as f32, optimal_action[1] as f32, optimal_action[2] as f32);
          let chosen_vec = Vec3::new(action[0] as f32, action[1] as f32, action[2] as f32);
          let dist = optimal_vec.distance(chosen_vec);          
          reward += ((1.0 - dist) as f64) * 0.01;
        }
        let uses_checkpoint = self.sim.config.with_checkpoint();
        if uses_checkpoint && !self.track_reached_checkpoint {
          if self.sim.config.checkpoint_range.vec3_in_range(&self.lines[self.lines.len()-1].0) {
            self.track_reached_checkpoint = true;
            if self.sim.config.reward_type.contains(&REWARD_TRACK_REACH_CHECKPOINT) {
              reward += 0.5;
            }
          }
        }
        if !self.track_reached_goal && ((uses_checkpoint && self.track_reached_checkpoint) || !uses_checkpoint) {
          if self.sim.config.goal_position.vec3_in_range(&self.lines[self.lines.len()-1].0) {
            self.track_reached_goal = true;
            if self.sim.config.reward_type.contains(&REWARD_GOAL_REACHED_BY_TRACK) {
              reward += 0.5;
            }
            if self.sim.config.reward_type.contains(&REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL) {

              self.current_step = self.sim.config.step_limit;
              is_done = true;
              reward += self.get_reward_through_simulation().0;
            }
          }
        }
      } else if self.sim.config.reward_type.contains(&REWARD_SCOLD_INVALID_ACTION) {
        reward = -0.0001;
      }
      self.current_step += 1;
      if !is_done && self.sim.config.reward_type.contains(&REWARD_SIMULATE_INBETWEEN) && self.current_step % self.sim.config.intermediate_simulation_frequency == 0 {
        reward += self.get_reward_through_simulation().0;
        self.reset_simulation_only();
      }
    }
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