pub mod gym;
pub mod tracks;
pub mod replayable;
use std::f64::consts::FRAC_PI_6;
use crate::{
  simulator::*,
  util::{calculate_euler_angles, consts::*, middle_of_two_points, radians_to_degree, track::{check_collision_with_colliders, get_all_points, get_change_vector_for_movement_in_direction, get_free_mesh_points, get_mesh_points, make_goal_range, MESH_INDICES_COLLIDER}}
};
use bevy::{math::Quat, prelude::{Vec3, Mut}};
use bevy_rapier3d::prelude::Collider;
use rusty_gym::{
  Observation, Space, util::rng::{UniRng, get_rng_for_type}
};
use ndarray::Array;
use rand::Rng;

pub struct LineRider3DEnv {
  pub rng: UniRng,
  pub action_space: Space,
  pub observation_space: Space,
  pub used_seed: u64,
  pub sim: LineRiderSim,
  pub lines: Vec<TrackPoint>,
  pub line_colliders: Vec<Collider>,
  pub current_direction: i64,
  pub prev_points: [Vec3; 4],
  pub current_step: usize,
  pub track_reached_goal: bool,
  pub track_reached_checkpoint: bool,
  pub track_distance_of_last_step: f32,
  pub skip_simulation: bool
}

impl std::fmt::Debug for LineRider3DEnv {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("LineRider3D-Env").finish()
  }
}

impl Default for LineRider3DEnv {
  fn default() -> LineRider3DEnv {
    let sim: LineRiderSim = LineRiderSim::default();
    LineRider3DEnv::new(sim, None)
  }
}

pub fn make_high_low(array_length: usize, half_width: f64) -> (Vec<f64>, Vec<f64>) {
  let mut low = Vec::with_capacity(array_length as usize);
  let mut high = Vec::with_capacity(array_length as usize);
  for i in 1..array_length+1 {
    low.push(if i > 0 && i%4 == 0 {0.0}else{-half_width});
    high.push(if i > 0 && i%4 == 0 {4.0}else{half_width});
  }
  (low, high)
}

impl LineRider3DEnv  {
  pub fn get_additional_points(config: &LineRiderConfig) -> usize {
    if config.with_checkpoint() {
      3
    } else {
      2
    }
  }
  pub fn get_observation_space(config: &LineRiderConfig) -> Space {
    let max_width = config.max_width as i64;
    let half_width = (config.max_width / 2.0) as f64;
    match config.observation_type {
      OBSERVATION_TYPE_3D_VIEW => { // TODO: limit numerical range
        Space::boxed(vec![max_width, max_width, max_width])
      },
      OBSERVATION_TYPE_3D_VIEW_ONEHOT => {
        // 5 instead of 4 because one is the goal
        Space::boxed(vec![max_width, max_width, max_width, 5])
      },
      OBSERVATION_TYPE_GOAL_AND_LAST_POINT => {
        Space::BoxedWithRange(vec![4, 2], vec![-half_width, -half_width, -half_width, 0.0, -half_width, -half_width, -half_width, 0.0], vec![half_width, half_width, half_width, 4.0, half_width, half_width, half_width, 4.0])
      },
      OBSERVATION_TYPE_SLIDING_WINDOW => { // OBSERVATION_TYPE_BUILD_POINTS +2 because start point of player & end / goal_point 4 instead of 3 because tracktype on top of xyz
        let array_length = (config.obs_sliding_window_size + 1) * 4;
        let (low, high) = make_high_low(array_length, 99.0); // 99 for better transfer between sizes
        Space::BoxedWithRange(vec![(config.obs_sliding_window_size + 1) as i64, 4], low, high)
      },
      _ => { // OBSERVATION_TYPE_BUILD_POINTS +2 because start point of player & end / goal_point 4 instead of 3 because tracktype on top of xyz
        let additional_points = LineRider3DEnv::get_additional_points(config);
        let array_length = (config.step_limit + additional_points) as i64 * 4;
        let (low, high) = make_high_low(array_length as usize, half_width);
        Space::BoxedWithRange(vec![(config.step_limit + additional_points) as i64, 4], low, high)
      }
    }
  }
  pub fn get_action_space(config: &LineRiderConfig) -> Space {
    let max_dist = config.max_piece_length as f64;
    match config.action_type {
      ACTION_TYPE_FREE_POINTS => {
        let half_width = (config.max_width / 2.0) as f64;
        Space::BoxedWithRange(vec![3], vec![-half_width, -half_width, -half_width], vec![half_width, half_width, half_width])
      },
      ACTION_TYPE_FREE_POINTS_RELATIVE => {
        Space::BoxedWithRange(vec![3], vec![-max_dist, -max_dist, -max_dist], vec![max_dist, max_dist, max_dist])
      },
      ACTION_TYPE_FREE_POINTS_WITH_TP | ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE => {
        Space::BoxedWithRange(vec![4], vec![-max_dist, -max_dist, -max_dist, 0.0], vec![max_dist, max_dist, max_dist, 4.0])
      },
      ACTION_TYPE_RADIAL => {
        Space::BoxedWithRange(vec![3], vec![0.00001, -FRAC_PI_6, -FRAC_PI_6], vec![max_dist, FRAC_PI_6, FRAC_PI_6])
      },
      ACTION_TYPE_RADIAL_WITH_TP => {
        Space::BoxedWithRange(vec![4], vec![0.00001, -FRAC_PI_6, -FRAC_PI_6, 0.0], vec![max_dist, FRAC_PI_6, FRAC_PI_6, 4.0])
      },
      ACTION_TYPE_STATIC_WITH_EMPTY => {
        Space::Discrete(12)
      },
      ACTION_TYPE_STATIC_WITH_BOOST => {
        Space::Discrete(24)
      },
      _ => {
        Space::Discrete(6) // ACTION_TYPE_STATIC
      }
    }
  }
  pub fn new(sim: LineRiderSim, seed: Option<u64>) -> LineRider3DEnv {
    let (rng, used_seed) = get_rng_for_type(&None, seed);
    let mut env: LineRider3DEnv = LineRider3DEnv {
      rng, used_seed,
      action_space: LineRider3DEnv::get_action_space(&sim.config),
      observation_space: LineRider3DEnv::get_observation_space(&sim.config),
      sim,
      lines: Vec::with_capacity(1000),
      line_colliders: Vec::with_capacity(1000),
      current_direction: DIRECTION_FORWARD,
      prev_points: PREV_POINTS_ZEROED,
      current_step: 0,
      track_reached_goal: false,
      track_reached_checkpoint: false,
      track_distance_of_last_step: 0.0,
      skip_simulation: false
    };
    env.reset_state();
    env
  }

  pub fn from_seed(seed: Option<u64>) -> LineRider3DEnv {
    let sim: LineRiderSim = LineRiderSim::default();
    LineRider3DEnv::new(sim, seed)
  }

  pub fn reset_simulation_only(&mut self) {
    self.sim.reset_state();
  }


  pub fn add_pipes(&mut self) {
  }

  pub fn gen_new_start(&mut self) -> Vec3 {
    let half_width = self.sim.config.max_width / 2.0;
    let x = self.rng.gen_range(-half_width..half_width);
    let y = if self.sim.config.use_new_goalgen {
      self.rng.gen_range(-half_width..half_width)
    } else {
      self.rng.gen_range(0.0..half_width)
    };
    let z = self.rng.gen_range(-half_width..half_width);
    Vec3::new(x, y, z)
  }

  pub fn gen_new_goal(&mut self) -> Vec3 {
    let half_width = self.sim.config.max_width / 2.0;
    let goal_pos =  self.lines[0].0;
    let goal_size =  self.sim.config.goal_size;
    let positive_max = half_width-goal_size;
    let min_distance = goal_size * 1.2;
    let x_mod = self.rng.gen_range(min_distance..goal_pos.x.abs() + positive_max);
    let x = if goal_pos.x > 0.0 {
      goal_pos.x - x_mod
    } else {
      goal_pos.x + x_mod
    };
    let y = match self.sim.config.target_type {
      TARGET_SAME_HEIGHT_AS_START | TARGET_RANDOM_WITH_CHECKPOINT_ABOVE | TARGET_RANDOM_WITH_CHECKPOINT_BELOW => {
        goal_pos.y
      },
      _ => {
        goal_pos.y - self.rng.gen_range(min_distance..positive_max+goal_pos.y.abs())
      }
    };
    let z_mod = self.rng.gen_range(min_distance..goal_pos.z.abs() + positive_max);
    let z = if goal_pos.z > 0.0 {
      goal_pos.z - z_mod
    } else {
      goal_pos.z + z_mod
    };
    Vec3::new(x, y, z)
  }

  pub fn generate_new_start_position(&mut self) {
    let half_width = self.sim.config.max_width / 2.0;
    let new_goal_gen = self.sim.config.use_new_goalgen;
    match self.sim.config.target_type { // TODO: make sure that some action is always executable!
      TARGET_STATIC_START_AND_END | TARGET_STATIC_START_RANDOM_END => {
        self.lines.push((Vec3::ZERO, TP_NORMAL));
      }
      _ => {
        let x = self.rng.gen_range(-half_width..half_width);
        let y = if new_goal_gen {self.rng.gen_range(-half_width..half_width)} else {
          match self.sim.config.target_type {
            TARGET_RANDOM_WITH_CHECKPOINT_ABOVE => {
              self.rng.gen_range(-half_width..0.0)
            },
            _ => {self.rng.gen_range(0.0..half_width)}
          }
        };
        let z = self.rng.gen_range(-half_width..half_width);
        self.lines.push((Vec3::new(x,y,z), TP_NORMAL));
      }
    }
    self.sim.origin = if new_goal_gen { Some(self.lines[0].0.clone()) } else  { None };
    self.sim.set_max_width(self.sim.config.max_width);
  }
  pub fn generate_new_goal_position(&mut self) {
    let half_width = self.sim.config.max_width / 2.0;
    // let half_half = half_width / 2.0;
    let new_goal_gen = self.sim.config.use_new_goalgen;
    match self.sim.config.target_type {
      TARGET_STATIC_START_AND_END | TARGET_RANDOM_START_STATIC_END => {
        self.sim.set_goal_position(Vec3::new(3.0, -3.0, 0.0));
      }
      _ => {
        if new_goal_gen {
          let goal_pos =  self.lines[0].0;
          let min_distance = half_width  / 2.0;
          let x = self.rng.gen_range(min_distance..half_width);
          let y = match self.sim.config.target_type {
            TARGET_ABOVE_START | TARGET_RANDOM_START_AND_END => {
              self.rng.gen_range(min_distance..half_width)
            },
            _ => {
              0.0
            }
          };
          let z = self.rng.gen_range(min_distance..half_width);
          let calculated_y = match self.sim.config.target_type {
            TARGET_RANDOM_START_AND_END => {goal_pos.y-y},
            TARGET_SAME_HEIGHT_AS_START | TARGET_RANDOM_WITH_CHECKPOINT_ABOVE | TARGET_RANDOM_WITH_CHECKPOINT_BELOW => {
              goal_pos.y
            },
            _ => {goal_pos.y+y}
          };
          let new_goal_pos = Vec3::new(goal_pos.x+x,calculated_y,goal_pos.z + z);
          self.sim.set_goal_position(new_goal_pos);
        } else {
          let mut is_in_start_range = true;
          while is_in_start_range {
            let new_goal_pos = self.gen_new_goal();
            self.sim.set_goal_position(new_goal_pos);
            is_in_start_range = self.sim.config.goal_position.vec3_in_range(&self.lines[0].0);
            if is_in_start_range {
              println!("need to regenerate goal because overlap");
            }
          }
        }
      }
    }
    if !new_goal_gen {
      if self.sim.config.target_type == TARGET_ABOVE_START { // TODO: verify this works
        let mut current_start = self.lines[0].0;
        let mut angle = calculate_euler_angles(current_start, self.sim.config.goal_pos).1;
        let mut is_in_start_range = false;
  
        while angle < self.sim.config.max_up_angle[0] || angle > self.sim.config.max_up_angle[1] || is_in_start_range {
          current_start = self.gen_new_start();
          let new_goal_pos = self.gen_new_goal();
          self.sim.set_goal_position(new_goal_pos);
          angle = calculate_euler_angles(current_start, self.sim.config.goal_pos).1;
  
          is_in_start_range = self.sim.config.goal_position.vec3_in_range(&self.lines[0].0);
        }
        self.lines.clear();
        self.lines.push((current_start, TP_NORMAL));
        let new_start = self.sim.config.goal_pos.clone();
        let new_goal_pos = self.lines[0].0;
        self.sim.set_goal_position(new_goal_pos);
        self.lines[0].0 = new_start;
      }
    }
    self.create_checkpoint();
    self.track_distance_of_last_step = self.sim.config.goal_pos.distance(self.lines[0].0);
  }

  pub fn reset_state(&mut self) {
    self.sim.reset_state();
    self.current_direction = DIRECTION_FORWARD;
    self.prev_points = PREV_POINTS_ZEROED;
    self.current_step = 0;
    self.track_reached_goal = false;
    self.track_reached_checkpoint = false;
    self.line_colliders.clear();
    self.lines.clear();
    self.generate_new_start_position();
    self.generate_new_goal_position();
    {
      let mut app_config: Mut<LineRiderConfig> = self.sim.app.world.resource_mut();
      app_config.copy_from(&self.sim.config);
    }
  }

  pub fn create_checkpoint(&mut self) {
    if self.sim.config.with_checkpoint() {
      let start = self.lines[0].0;
      let end = self.sim.config.goal_pos;
      let mut middle = middle_of_two_points(&start, &end);
      match self.sim.config.target_type {
        TARGET_RANDOM_WITH_CHECKPOINT_ABOVE => {
          let min = start.y + 0.5;
          let top = self.sim.build_range.y_max - 0.5;
          middle.y = self.rng.gen_range(min..top);
        },
        TARGET_RANDOM_WITH_CHECKPOINT_BELOW => {
          let min = self.sim.build_range.y_min + 0.5;
          let top = start.y - 0.5;
          middle.y = self.rng.gen_range(min..top);
        },
        _ => {}
      }
      self.sim.config.checkpoint_range = make_goal_range(&middle, &self.sim.config);
      self.sim.config.checkpoint_pos = middle;
      {
        let mut app_config: Mut<LineRiderConfig> = self.sim.app.world.resource_mut();
        app_config.copy_from(&self.sim.config);
      }
    }
  }
  pub fn make_obs(&self) -> Observation {
    let max_width = self.sim.config.max_width as usize;
    let half_width = max_width as f32 / 2.0;
    let goal_pos = &self.sim.config.goal_pos;
    match self.sim.config.observation_type {
      OBSERVATION_TYPE_3D_VIEW => {
        let mut obs = Array::from_elem((max_width, max_width, max_width), -1.0 as f64);
        obs[((goal_pos.x + half_width) as usize, (goal_pos.y + half_width) as usize, (goal_pos.z + half_width) as usize)] = TP_GOAL as f64;
        for line in &self.lines {
          obs[((line.0.x + half_width) as usize, (line.0.y + half_width) as usize, (line.0.z + half_width) as usize)] = line.1 as f64;
        }
        obs.into_dyn()
      },
      OBSERVATION_TYPE_3D_VIEW_ONEHOT => {
        let mut obs = Array::from_elem((max_width, max_width, max_width, 5), 0.0);
        for line in &self.lines {
          obs[((line.0.x + half_width) as usize, (line.0.y + half_width) as usize, (line.0.z + half_width) as usize, line.1 as usize)] = 1.0;
        }
        obs[((goal_pos.x + half_width) as usize, (goal_pos.y + half_width) as usize, (goal_pos.z + half_width) as usize, (TP_DECELERATE+1) as usize)] = 1.0;
        obs.into_dyn()
      },
      OBSERVATION_TYPE_GOAL_AND_LAST_POINT => {
        let mut obs = Array::from_elem((2, 4), -1.0);
        let line = self.lines[self.lines.len()-1];
        let i = 0;
        obs[[i, 0]] = line.0.x as f64;
        obs[[i, 1]] = line.0.y as f64;
        obs[[i, 2]] = line.0.z as f64;
        obs[[i, 3]] = line.1 as f64;
        let i = 1;
        obs[[i, 0]] = goal_pos.x as f64;
        obs[[i, 1]] = goal_pos.y as f64;
        obs[[i, 2]] = goal_pos.z as f64;
        obs[[i, 3]] = TP_GOAL as f64;
        obs.into_dyn()
      },
      OBSERVATION_TYPE_SLIDING_WINDOW => {
        let mut obs = Array::from_elem((self.sim.config.obs_sliding_window_size+1, 4), -1.0 as f64);
        let mut latest_index = 0;
        'INDEX_SEARCH: for i in 0..self.lines.len() {
          latest_index = self.lines.len()-i-1;
          let point = self.lines[latest_index].0;
          if point != Vec3::NEG_ONE {
            break 'INDEX_SEARCH;
          }
        }
        // [1, 2, 3, 4, 5, 6, -1]
        for i in 0..self.sim.config.obs_sliding_window_size {
          let index_to_add = latest_index as i64 - i as i64;
          if index_to_add >= 0 {
            let line: (Vec3, u8) = self.lines[index_to_add as usize];
            let add_at = self.sim.config.obs_sliding_window_size-i-1;
            obs[[add_at, 0]] = line.0.x as f64;
            obs[[add_at, 1]] = line.0.y as f64;
            obs[[add_at, 2]] = line.0.z as f64;
            obs[[add_at, 3]] = line.1 as f64;
          }
        }
        let i = self.sim.config.obs_sliding_window_size;
        let current_target = if self.sim.config.with_checkpoint() && !self.track_reached_checkpoint {
          &self.sim.config.checkpoint_pos
        } else {
          goal_pos
        };
        obs[[i, 0]] = current_target.x as f64;
        obs[[i, 1]] = current_target.y as f64;
        obs[[i, 2]] = current_target.z as f64;
        obs[[i, 3]] = TP_GOAL as f64;
        obs.into_dyn()
      },
      _ => { // OBSERVATION_TYPE_BUILD_POINTS
          let additional_points = LineRider3DEnv::get_additional_points(&self.sim.config);
          let obs_len = (self.sim.config.step_limit + additional_points);
          let mut obs = Array::from_elem((obs_len, 4), -1.0);
          let with_checkpoint = self.sim.config.with_checkpoint();
          let half_len: usize = self.sim.config.step_limit / 2;
          for i in 0..self.lines.len() {
            let line = self.lines[i];
            obs[[i, 0]] = line.0.x as f64;
            obs[[i, 1]] = line.0.y as f64;
            obs[[i, 2]] = line.0.z as f64;
            obs[[i, 3]] = line.1 as f64;
          }
          if with_checkpoint {
            for i in half_len..self.sim.config.step_limit {
              if i < self.lines.len() {
                obs[[i+1, 0]] = self.lines[i].0.x as f64;
                obs[[i+1, 1]] = self.lines[i].0.y as f64;
                obs[[i+1, 2]] = self.lines[i].0.z as f64;
                obs[[i+1, 3]] = self.lines[i].1 as f64;
              }
            }
            let checkpoint_pos = &self.sim.config.checkpoint_pos;
            obs[[half_len, 0]] = checkpoint_pos.x as f64;
            obs[[half_len, 1]] = checkpoint_pos.y as f64;
            obs[[half_len, 2]] = checkpoint_pos.z as f64;
            obs[[half_len, 3]] = TP_GOAL as f64;
          }
          let i = obs_len-1;
          obs[[i, 0]] = goal_pos.x as f64;
          obs[[i, 1]] = goal_pos.y as f64;
          obs[[i, 2]] = goal_pos.z as f64;
          obs[[i, 3]] = TP_GOAL as f64;
          obs.into_dyn()
      }
    }
  }

  pub fn add_lines(&mut self) {
    let mut track_to_add: Mut<TrackToAdd> = self.sim.app.world.resource_mut();
    track_to_add.0 = self.lines.clone();
    track_to_add.1 = false;
  }

  pub fn add_lines_freeroam(&mut self) {
    let mut track_to_add: Mut<TrackToAdd> = self.sim.app.world.resource_mut();
    track_to_add.0 = self.lines.clone();
    track_to_add.1 = true;
  }
  pub fn add_line_for_point(&mut self, track_point: TrackPoint) -> bool {
    let prev_point = self.lines[self.lines.len()-1].0;
    let current_point = track_point.0;

    // TODO: this conflicts with radial coordinates and might have never done what it was supposed to
    // let angle = prev_point.angle_between(current_point);
    // if radians_to_degree(angle) > 50.0 {
    //   println!("ANGLE TO LARGE");
    //   return false;
    // }
    let (all_points, _) = get_free_mesh_points(prev_point, current_point, &self.prev_points, &self.sim.config);
    self.prev_points = [all_points[1][1], all_points[2][1], all_points[3][1], all_points[4][1]];

    if self.sim.build_range.vec3_in_range(&track_point.0) {
      if track_point.1 == TP_EMPTY || self.sim.config.skip_collision_check_on_last_x_pieces == 999 {
        self.lines.push(track_point);
        true
      } else {
        self.add_tp_with_collision_check(&all_points, track_point, true)
      }
    } else {
      false
    }
  }
  /**
   * returns true if action was valid and executed
   * returns false if track part was not added due to e.g. collision
   */
  pub fn add_line_for_action(&mut self, mut action_val: i64) -> bool {
    let mut track_type = TP_NORMAL;
    let is_empty = action_val > 5 && action_val < 12;
    if is_empty {
      action_val = action_val - 6;
      track_type = TP_EMPTY;
    } else if action_val > 17 {
      action_val = action_val - 18;
      track_type = TP_DECELERATE;
    } else if action_val > 11 {
      action_val = action_val - 12;
      track_type = TP_ACCELERATE;
    }
    let last_position = self.lines[self.lines.len()-1].0;
    let (change_by, new_direction) = get_change_vector_for_movement_in_direction(self.current_direction, action_val, self.sim.config.track_piece_length);
    let new_position = last_position + change_by;
    let track_point = (new_position, track_type);
    if self.sim.build_range.vec3_in_range(&new_position) {
      if !is_empty {
        let all_points = get_all_points(last_position, new_position, self.current_direction, new_direction, action_val, &self.sim.config);
        if self.add_tp_with_collision_check(&all_points, track_point, false) {
          self.current_direction = new_direction;
          return true;
        } else {
          return false;
        }
      } else {
        self.current_direction = new_direction;
        self.lines.push(track_point);
        true
      }
    } else {
      false
    }
  }
  pub fn add_tp_with_collision_check(&mut self, all_points: &[Vec<Vec3>], new_point: TrackPoint, free_points: bool) -> bool {
    let mut has_collision = false;
    let mut new_colliders: Vec<Collider> = Vec::with_capacity(self.sim.config.bezier_resolution);
    let max = if free_points {2} else {all_points.len()};
    let (amount_of_colliders_to_skip, prediction): (usize, Option<f32>) = match self.sim.config.action_type {
      ACTION_TYPE_STATIC | ACTION_TYPE_STATIC_WITH_EMPTY  | ACTION_TYPE_STATIC_WITH_BOOST => {(1, Some(0.09))},
      ACTION_TYPE_RADIAL | ACTION_TYPE_RADIAL_WITH_TP => {(2, Some(0.01))},
      _ => {(self.sim.config.skip_collision_check_on_last_x_pieces, Some(0.0))}
    };
    'COL_CHECK: for u in 1..max {
      let mesh_points = get_mesh_points(&all_points, u);

      let cols_to_add = vec![
        // Collider::trimesh(mesh_points.clone(), MESH_INDICES_TRACK_COL.to_vec()),
        // Collider::trimesh(mesh_points.clone(), MESH_INDICES_LEFT_WALL.to_vec()),
        // Collider::trimesh(mesh_points, MESH_INDICES_RIGHT_WALL.to_vec()),
        Collider::trimesh(mesh_points, MESH_INDICES_COLLIDER.to_vec()),
      ];
      for new_collider in cols_to_add {
        if check_collision_with_colliders(&self.line_colliders, &new_collider, amount_of_colliders_to_skip, prediction) {
          has_collision = true;
          break 'COL_CHECK;
        }
        new_colliders.push(new_collider);
      }
      
    }
    if has_collision || self.lines.iter().position(|tp| tp.0 == new_point.0).is_some() {
      false
    } else {
      self.line_colliders.append(&mut new_colliders);
      self.lines.push(new_point);
      true
    }
  }
  pub fn get_reward_from_simulation_result(&mut self, sim_res: LineRiderSimulationResult) -> f64 {
    let mut reward: f64 = 0.0;
    let LineRiderSimulationResult {steps_taken, goal_reached,
      velocity_at_end, overall_velocity, checkpoint_reached,
      overall_rotation, overall_height_gain,
      closest_to_goal, ended_because_of_no_movement,
      time_rider_touched_track, time_rider_airborne, total_time } = sim_res;
    let uses_checkpoint = self.sim.config.with_checkpoint();
    for reward_type in &self.sim.config.reward_type {
      match *reward_type {
        REWARD_GOAL_REACHED_BY_BALL => {
          let second_requirement_fulfilled = if uses_checkpoint {
            checkpoint_reached
          } else { true };
          if second_requirement_fulfilled && goal_reached {reward += 2.0;}
        },
        REWARD_REACH_CHECKPOINT => {
          if checkpoint_reached {reward += 2.0;}
        },
        REWARD_GOAL_REACHED_BY_BOTH_ONLY => {
          if goal_reached && self.track_reached_goal {reward += 2.0;}
        },
        REWARD_FASTEST_GOAL_REACH => {
          if goal_reached {
            reward += ((self.sim.config.simulation_steps / steps_taken) / self.sim.config.simulation_steps) as f64;
          }
        },
        REWARD_DISTANCE_TO_GOAL_IN_SIMULATION => {
          if !checkpoint_reached || !goal_reached {reward += (closest_to_goal / 1000.0) as f64;};
        },
        REWARD_DISTANCE_TO_GOAL_IN_SIMULATION_IF_TRACK_REACHED_GOAL => {
          if !goal_reached && self.track_reached_goal {reward += (closest_to_goal / 1000.0) as f64;};
        },
        REWARD_GOING_UP => {reward += (overall_height_gain / 1000.0) as f64;},
        REWARD_LONGEST_TRACK => {reward += (self.lines.len() as f64 / self.sim.config.step_limit as f64) * 0.25;},
        REWARD_SHORTEST_TRACK => {reward += ((self.sim.config.step_limit as f64 / self.lines.len() as f64) / self.sim.config.step_limit as f64) * 0.25;},
        REWARD_SPEED_TOTAL => {reward += overall_velocity as f64 / (self.sim.config.simulation_steps * 2) as f64;},
        REWARD_SPEED_AT_END => {reward += velocity_at_end as f64 / 50.0;},
        REWARD_LOW_SPEED_AT_END => {reward += (50.0 / velocity_at_end as f64) / 50.0;},
        REWARD_MOST_ROTATION => {reward += overall_rotation as f64 / self.sim.config.simulation_steps as f64;},
        REWARD_LEAST_ROTATION => {reward += (self.sim.config.simulation_steps as f64 /  overall_rotation as f64) / self.sim.config.simulation_steps as f64;},
        REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END => {
          let has_reached = if uses_checkpoint && !checkpoint_reached {checkpoint_reached} else {goal_reached};
          if !has_reached && !self.track_reached_goal {
            let pos_to_use = if uses_checkpoint && !checkpoint_reached {self.sim.config.checkpoint_pos} else {self.sim.config.goal_pos};
            let dist = pos_to_use.distance(self.lines[self.lines.len()-1].0);
            if dist < (self.sim.config.max_width / 2.0) {
              reward += ((1.0 - (dist / (self.sim.config.max_width / 2.0))) / 10.0) as f64;
            }
          }
        },
        REWARD_SCOLD_PREMATURE_END => {if ended_because_of_no_movement {reward -= 1.0}},
        REWARD_TRACK_TOUCHES => {
          reward += (time_rider_touched_track as f64 / total_time as f64) * 0.25;
        },
        REWARD_AIR_TIME => {
          reward += (time_rider_airborne as f64 / total_time as f64) * 0.25;
        }
        _ => {}
      }
    }
    reward
  }
  pub fn get_reward_through_simulation(&mut self) -> (f64, LineRiderSimulationResult) {
    if self.skip_simulation {
      return (0.0, LineRiderSimulationResult::default());
    }
    match self.sim.config.action_type {
      ACTION_TYPE_FREE_POINTS | ACTION_TYPE_FREE_POINTS_WITH_TP |
        ACTION_TYPE_FREE_POINTS_RELATIVE | ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE => {
        self.add_lines_freeroam();
      }
      _ => {
        self.add_lines();
      }
    };
    let sim_res = self.sim.simulate_till_end(self.sim.config.simulation_steps);
    let reward = self.get_reward_from_simulation_result(sim_res.clone());
    (reward, sim_res)
  }
}



#[cfg(test)]
mod tests {
  use insta::assert_debug_snapshot;
  use ndarray::ArrayBase;
  use crate::{util::{range::Range3D, degree_to_radians}, algo::heuristic::straight_line_heuristic_general, env::tracks::prepare_acc_jump};
  use super::*;
  use crate::env::tracks::{prepare_curvy_track, prepare_ramp_track};
  use rusty_gym::{GymEnvironment, ReplayableGymEnvironment};
  
  #[test]
  fn ball_reaches_bottom_of_circle() {
    // let sim: LineRiderSim = LineRiderSim::new();
    let mut env: LineRider3DEnv = LineRider3DEnv::default();
    env.sim.set_max_width(50.0);
    env.sim.config.target_type = TARGET_STATIC_START_AND_END;
    let new_fake_time = 140;
    env.sim.config.physics_delta = new_fake_time;
    env.sim.set_fake_delta(new_fake_time);
    env.sim.set_physics_delta(new_fake_time, 1);
    let steps_needed_for_one_second = 1000/new_fake_time;
    
    env.sim.set_goal_position(Vec3::new(0.0, -8.5, 0.0));
    prepare_curvy_track(&mut env);
    env.add_lines();

    let result_tuple = env.sim.simulate_till_end((steps_needed_for_one_second*100) as usize);
    assert_debug_snapshot!("bottom_circle_reached", result_tuple);
  }

  #[test]
  fn ramp_jump_reaches_unconnected_track() {
    // let sim: LineRiderSim = LineRiderSim::new();
    let mut env: LineRider3DEnv = LineRider3DEnv::default();
    env.sim.set_max_width(50.0);
    let new_fake_time = 80;
    env.sim.config.physics_delta = new_fake_time;
    env.sim.set_fake_delta(new_fake_time);
    env.sim.set_physics_delta(new_fake_time, 1);
    let steps_needed_for_one_second = 1000 / new_fake_time;
    env.sim.config.step_limit = 10;
    assert_debug_snapshot!("obs_history_empty", env.make_obs());
    prepare_ramp_track(&mut env);
    env.sim.set_goal_position(env.lines[env.lines.len()-1].0 + 0.4);
    assert_debug_snapshot!("obs_history_filled", env.make_obs());
    env.sim.config.observation_type = OBSERVATION_TYPE_3D_VIEW;
    env.sim.config.max_width = 21.0;
    assert_debug_snapshot!("obs_3D", env.make_obs());
    env.sim.config.observation_type = OBSERVATION_TYPE_3D_VIEW_ONEHOT;
    assert_debug_snapshot!("obs_3D_onehot", env.make_obs());
    env.sim.config.observation_type = OBSERVATION_TYPE_GOAL_AND_LAST_POINT;
    assert_debug_snapshot!("obs_goal_lastpoint_only", env.make_obs());
    env.sim.config.observation_type = OBSERVATION_TYPE_SLIDING_WINDOW;
    assert_debug_snapshot!("obs_sliding_window", env.make_obs());
    env.add_lines();

    let result_tuple = env.sim.simulate_till_end((steps_needed_for_one_second*100) as usize);
    assert_debug_snapshot!("ramp_jump", result_tuple);

    for i in 0..9 {
      env.lines.pop();
    }
    env.sim.config.observation_type = OBSERVATION_TYPE_SLIDING_WINDOW;
    assert_debug_snapshot!("obs_sliding_window_almost_empty", env.make_obs());
  }

  // #[test]
  // fn prevent_colliding_tracks() {
  //   let mut env: LineRider3DEnv = LineRider3DEnv::default();
  //   env.sim.set_max_width(50.0);
  //   env.sim.config.target_type = TARGET_STATIC_START_AND_END;
  //   // prevent simple loops
  //   for action in [ACTION_LEFT, ACTION_RIGHT] {
  //     env.reset();
  //     for i in 0..4 {
  //       assert_eq!(env.add_line_for_action(action), if i == 3 {false} else {true});
  //     }
  //   }
  //   // Prevent track wall to collide with previous trackparts
  //   env.reset();
  //   assert_eq!(env.add_line_for_action(ACTION_DOWN), true);
  //   assert_eq!(env.add_line_for_action(ACTION_LEFT), true);
  //   assert_eq!(env.add_line_for_action(ACTION_LEFT), true);
  //   assert_eq!(env.add_line_for_action(ACTION_LEFT), true);
  //   assert_eq!(env.add_line_for_action(ACTION_LEFT), false);
  //   assert_eq!(env.add_line_for_action(ACTION_STRAIGHT), false);
  //   assert_eq!(env.add_line_for_action(ACTION_UP), false);
  //   assert_eq!(env.add_line_for_action(ACTION_RIGHT), false);
  //   // works currently but collision detection might be too broad and this needs to be reverted by adjusting distance in contact query
  //   assert_eq!(env.add_line_for_action(ACTION_DOWN), true);
  //   env.reset();
  //   env.sim.set_max_width(4.0);
  //   assert_eq!(env.add_line_for_action(ACTION_STRAIGHT), true);
  //   assert_eq!(env.add_line_for_action(ACTION_STRAIGHT), false);

  //   env.reset();
  //   env.add_line_for_point((Vec3::new(-0.88962376, 2.7135723, -0.96368843), 1));
  //   env.add_line_for_point((Vec3::new(1.8226265, 2.4700809, 0.6920514), 1));
  //   env.add_line_for_point((Vec3::new(0.5565916, -0.15580577, 1.1010047), 1));
  //   env.add_line_for_point((Vec3::new(0.10373199, 0.3419075, -0.24062479), 1));

  //   env.sim.config.target_type = TARGET_RANDOM_START_AND_END;
  //   env.sim.set_goal_position(Vec3::new(3.0, 1.0, -1.0));
  //   env.sim.set_max_width(10.0);
  //   env.use_seed(42);
  //   env.reset();
  //   env.add_line_for_point((Vec3::new(-0.88962376, 2.7135723, -0.96368843), 1));
  //   env.add_line_for_point((Vec3::new(0.88962376, -2.7135723, 0.96368843), 1));
  //   env.add_line_for_point((Vec3::new(-0.98962376, 2.7135723, -0.96368843), 1));

  // }

  #[test]
  fn build_track_via_actions() {
    let mut env: LineRider3DEnv = LineRider3DEnv::default();
    env.use_seed(42);
    env.sim.reset_state();
    env.sim.set_max_width(50.0);
    env.sim.config.goal_position = Range3D::new(9.5, 11.0, -11.0, -9.5, -2.0, 2.0);
    let new_fake_time = 80;
    env.sim.config.physics_delta = new_fake_time;
    // env.sim.set_fake_delta(new_fake_time);
    env.sim.set_physics_delta(new_fake_time, new_fake_time as usize / 33);
    let steps_needed_for_one_second = 1000/new_fake_time;
    
    for _ in 0..10 {
      env.step(&ArrayBase::from(vec![ACTION_DOWN as f64]).into_dyn());
    }
    env.add_lines();

    let mut snaps = Vec::new();
    // snaps.push(env.sim.get_driver_transform().clone());
    let mut sim_second = || {
      for _ in 0..steps_needed_for_one_second {
        env.sim.simulation_step();
      }
      snaps.push(env.sim.get_driver_transform().clone());
    };
    for _ in 0..10 {
      sim_second();
    }

    snaps.push(env.sim.get_driver_transform().clone());

    env.sim.config.target_type = TARGET_STATIC_START_AND_END;
    env.reset();
    env.sim.set_goal_position(Vec3::new(10.25, -10.25, 0.0));
    for _ in 0..10 {
      env.step(&ArrayBase::from(vec![ACTION_DOWN as f64]).into_dyn());
    }
    env.add_lines();

    let result_tuple = env.sim.simulate_till_end((steps_needed_for_one_second*10) as usize);
    assert_debug_snapshot!("simulationresult", result_tuple);
    snaps.push(env.sim.get_driver_transform().clone());
    assert_debug_snapshot!("Steps", snaps);

    
    env.reset();
  }
  #[test]
  fn radial_action_space() {
    let mut sim: LineRiderSim = LineRiderSim::default();
    sim.config.action_type = ACTION_TYPE_RADIAL;
    let mut env: LineRider3DEnv = LineRider3DEnv::new(sim, None);
    env.use_seed(42);
    env.sim.reset_state();
    env.sim.set_max_width(10.0);
    
    // UP
    env.step(&ArrayBase::from(vec![0.5, degree_to_radians(0.0) as f64, degree_to_radians(-12.0) as f64]).into_dyn());
    // // DOWN
    env.step(&ArrayBase::from(vec![1.5, degree_to_radians(-5.0) as f64, degree_to_radians(0.0) as f64]).into_dyn());
    // // DOWN
    env.step(&ArrayBase::from(vec![0.2, degree_to_radians(5.0) as f64, degree_to_radians(12.0) as f64]).into_dyn());
    // LEFT
    env.step(&ArrayBase::from(vec![0.3, degree_to_radians(15.0) as f64, degree_to_radians(0.0) as f64]).into_dyn());
    
    env.add_lines();
    assert_debug_snapshot!("radial_lines", env.lines);
  }
  #[test]
  fn mimic_heuristic() {
    let sim: LineRiderSim = LineRiderSim::default();
    let mut env: LineRider3DEnv = LineRider3DEnv::new(sim, None);
    env.use_seed(42);
    env.sim.set_max_width(10.0);
    env.sim.config.action_type = ACTION_TYPE_FREE_POINTS_RELATIVE;
    env.sim.config.target_type = TARGET_RANDOM_START_AND_END;
    env.sim.set_goal_position(Vec3::new(3.0, 1.0, -1.0));
    env.sim.set_max_width(10.0);
    env.sim.config.reward_type = vec![REWARD_MIMIC_STRAIGHT_LINE_HEURISTIC];
    let step_res = 
    env.step(&ArrayBase::from(vec![0.90453404, 0.30151135, -0.30151135]).into_dyn());
    assert_eq!(step_res.reward, 0.01);
    let step_res = 
    env.step(&ArrayBase::from(vec![1.74886715, -1.48933995, 2.44692776]).into_dyn());
    assert!(step_res.reward < 0.0);
  }
  #[test]
  fn distance_reward() {
    let mut sim: LineRiderSim = LineRiderSim::default();
    sim.config.action_type = ACTION_TYPE_STATIC;
    let mut env: LineRider3DEnv = LineRider3DEnv::new(sim, None);
    env.use_seed(42);
    env.sim.reset_state();
    env.sim.set_goal_position(Vec3::new(0.0, 0.0, 4.0));
    env.sim.set_max_width(10.0);
    env.lines[0] = (Vec3::ZERO, 1);
    env.sim.config.reward_type = vec![REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END, REWARD_TRACK_CLOSER_TO_GOAL_IN_STEP];
    let reward = env.get_reward_through_simulation().0;
    assert!(reward > 0.015);
    let step_res = 
    env.step(&ArrayBase::from(vec![ACTION_STRAIGHT as f64]).into_dyn());
    assert_eq!(step_res.reward, 0.01);
    env.step(&ArrayBase::from(vec![ACTION_STRAIGHT as f64]).into_dyn());
    assert_eq!(step_res.reward, 0.01);
    let reward = env.get_reward_through_simulation().0;
    assert!(reward < 0.015);
  }
  // #[test]
  // fn premature_end() {
  //   let mut sim: LineRiderSim = LineRiderSim::default();
  //   sim.config.action_type = ACTION_TYPE_STATIC;
  //   let mut env: LineRider3DEnv = LineRider3DEnv::new(sim, None);
  //   env.use_seed(42);
  //   env.sim.reset_state();
  //   env.sim.set_goal_position(Vec3::new(0.0, 0.0, 4.0));
  //   env.sim.set_max_width(10.0);
  //   env.lines[0] = (Vec3::ZERO, 1);
  //   env.step(&ArrayBase::from(vec![ACTION_STRAIGHT_DOWN as f64]).into_dyn());
  //   env.add_lines();
  //   let max_steps = 9999999999;
  //   let res = env.sim.simulate_till_end(max_steps);
  //   assert!(res.steps_taken < max_steps-100);
  //   println!("Ended after {} steps", res.steps_taken);
  // }


  #[test]
  fn boost_tracks() {
    // let sim: LineRiderSim = LineRiderSim::new();
    let mut env: LineRider3DEnv = LineRider3DEnv::default();
    env.sim.set_max_width(50.0);
    let new_fake_time = 80;
    env.sim.config.physics_delta = new_fake_time;
    env.sim.set_fake_delta(new_fake_time);
    env.sim.set_physics_delta(new_fake_time, 1);
    let steps_needed_for_one_second = 1000 / new_fake_time;
    env.sim.config.step_limit = 10;
    prepare_acc_jump(&mut env);
    env.add_lines();

    let result_tuple = env.sim.simulate_till_end((steps_needed_for_one_second*100) as usize);
    assert_debug_snapshot!("boost_jump", result_tuple);
  }
  #[test]
  fn checkpoint_reachable() {
    let mut sim: LineRiderSim = LineRiderSim::default();
    sim.config.action_type = ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE;
    sim.config.target_type = TARGET_RANDOM_WITH_CHECKPOINT_BELOW;
    let mut env: LineRider3DEnv = LineRider3DEnv::new(sim, None);
    env.sim.config.step_limit = 10;
    env.skip_simulation = true;
    env.use_seed(42);
    let config = env.get_config();
    let mut obs: Observation = env.reset();
    assert_debug_snapshot!("checkpoint_obs_start", obs);
    for _ in 0..10 {
      let res = env.step(&straight_line_heuristic_general(&obs, &config, None));
      obs = res.obs;
    }
    assert_debug_snapshot!("checkpoint_obs", env.make_obs());
    env.add_lines_freeroam();
    let result_tuple = env.sim.simulate_till_end(9999);
    assert_debug_snapshot!("checkpoint_res", result_tuple);
  }
}
