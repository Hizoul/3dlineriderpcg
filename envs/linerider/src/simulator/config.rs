use bevy::prelude::{Vec3, Resource};
use crate::util::{range::Range3D, consts::*};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct LineRiderSimulationResult {
  pub steps_taken: usize,
  pub goal_reached: bool,
  pub checkpoint_reached: bool,
  pub velocity_at_end: f32,
  pub overall_velocity: f32,
  pub overall_rotation: f32,
  pub overall_height_gain: f32,
  pub closest_to_goal: f32,
  pub ended_because_of_no_movement: bool,
  pub time_rider_touched_track: u64,
  pub time_rider_airborne: u64,
  pub total_time: u64
}

impl LineRiderSimulationResult {
  pub fn to_map(&self) -> HashMap<String, String> {
    let mut additional_info: HashMap<String, String> = HashMap::new();
    additional_info.insert("checkpoint_reached".to_owned(), self.checkpoint_reached.to_string());
    additional_info.insert("total_time".to_owned(), self.total_time.to_string());
    additional_info.insert("velocity_at_end".to_owned(), self.velocity_at_end.to_string());
    additional_info.insert("overall_velocity".to_owned(), self.overall_velocity.to_string());
    additional_info.insert("overall_rotation".to_owned(), self.overall_rotation.to_string());
    additional_info.insert("overall_height_gain".to_owned(), self.overall_height_gain.to_string());
    additional_info.insert("closest_to_goal".to_owned(), self.closest_to_goal.to_string());
    additional_info.insert("ended_because_of_no_movement".to_owned(), self.ended_because_of_no_movement.to_string());
    additional_info.insert("time_rider_touched_track".to_owned(), self.time_rider_touched_track.to_string());
    additional_info.insert("time_rider_airborne".to_owned(), self.time_rider_airborne.to_string());
    additional_info.insert("total_time".to_owned(), self.total_time.to_string());
    additional_info.insert("steps_taken".to_owned(), self.total_time.to_string());
    additional_info
  }
}

#[derive(Clone, Resource)]
pub struct LineRiderConfig {
  // Simulation Settings
  pub max_width: f32,
  pub max_height: f32,
  pub rider_shape: usize,
  pub rider_mass: f32,
  pub rider_density: f32,
  pub rider_size: Vec3,
  pub track_width: f32,
  pub track_wall_width: f32,
  pub track_wall_height: f32,
  pub track_piece_length: f32,
  pub starting_force_multiplier: f32,
  pub bezier_resolution: usize,
  pub smooth_free_points: bool,
  pub goal_position: Range3D<f32>,
  pub goal_size: f32,
  pub goal_pos: Vec3,
  pub checkpoint_pos: Vec3,
  pub checkpoint_range: Range3D<f32>,
  pub premature_end_min_distance: f32,
  pub premature_end_after_steps_without_movement: usize,
  pub booster_strength: f32,
  pub use_cylinder_track: bool,
  // Env settings
  pub observation_type: u8,
  pub action_type: u8,
  pub reward_type: Vec<u8>,
  pub target_type: u8,
  pub step_limit: usize,
  pub simulation_steps: usize,
  pub max_piece_length: f32,
  pub physics_delta: u64,
  pub physics_substeps: usize,
  pub intermediate_simulation_frequency: usize,
  pub skip_collision_check_on_last_x_pieces: usize,
  pub max_up_angle: Vec<f32>,
  pub obs_sliding_window_size: usize,
  pub use_new_goalgen: bool
}

impl LineRiderConfig {
  pub fn with_checkpoint(&self) -> bool {
    match self.target_type {
      TARGET_RANDOM_WITH_CHECKPOINT_ABOVE | TARGET_RANDOM_WITH_CHECKPOINT_BELOW => true,
      _ => false
    }
  }
  pub fn copy_from(&mut self, other_conf: &LineRiderConfig) {
    self.max_width = other_conf.max_width;
    self.max_height = other_conf.max_height;
    self.rider_shape = other_conf.rider_shape;
    self.rider_mass = other_conf.rider_mass;
    self.rider_density = other_conf.rider_density;
    self.rider_size = other_conf.rider_size;
    self.track_width = other_conf.track_width;
    self.track_wall_width = other_conf.track_wall_width;
    self.track_wall_height = other_conf.track_wall_height;
    self.track_piece_length = other_conf.track_piece_length;
    self.starting_force_multiplier = other_conf.starting_force_multiplier;
    self.bezier_resolution = other_conf.bezier_resolution;
    self.smooth_free_points = other_conf.smooth_free_points;
    self.goal_position = other_conf.goal_position.clone();
    self.goal_pos = other_conf.goal_pos;
    self.checkpoint_pos = other_conf.checkpoint_pos;
    self.checkpoint_range = other_conf.checkpoint_range.clone();
    self.premature_end_min_distance = other_conf.premature_end_min_distance;
    self.premature_end_after_steps_without_movement = other_conf.premature_end_after_steps_without_movement;
    self.booster_strength = other_conf.booster_strength;
    self.observation_type = other_conf.observation_type;
    self.action_type = other_conf.action_type;
    self.reward_type = other_conf.reward_type.clone();
    self.target_type = other_conf.target_type;
    self.step_limit = other_conf.step_limit;
    self.simulation_steps = other_conf.simulation_steps;
    self.max_piece_length = other_conf.max_piece_length;
    self.physics_delta = other_conf.physics_delta;
    self.physics_substeps = other_conf.physics_substeps;
    self.intermediate_simulation_frequency = other_conf.intermediate_simulation_frequency;
    self.skip_collision_check_on_last_x_pieces = other_conf.skip_collision_check_on_last_x_pieces;
    self.use_cylinder_track = other_conf.use_cylinder_track;
    self.max_up_angle = other_conf.max_up_angle.clone();
    self.obs_sliding_window_size = other_conf.obs_sliding_window_size;
    self.use_new_goalgen = other_conf.use_new_goalgen;
  }
}

impl Default for LineRiderConfig {
  fn default() -> LineRiderConfig {
    LineRiderConfig {
      max_width: 10.0,
      max_height: 5.0,
      rider_shape: 1,
      rider_mass: 1.0,
      rider_density: 5.0,
      rider_size: Vec3::new(0.5, 0.125, 0.25),
      track_width: 0.5,
      track_wall_width: 0.5,
      track_wall_height: 0.75,
      track_piece_length: 1.0,
      max_piece_length: 1.5,
      starting_force_multiplier: 1.2,
      skip_collision_check_on_last_x_pieces: 999,
      bezier_resolution: 5,
      smooth_free_points: false,
      goal_position: Range3D::new(10.0, 20.0, -6.0, -2.0, 3.0, 7.0),
      goal_size: 1.25,
      goal_pos: Vec3::new(0.5, 0.125, 0.25),
      checkpoint_pos: Vec3::new(-999.0, -999.0, -999.0),
      checkpoint_range: Range3D::default(),
      observation_type: 0,
      action_type: 0,
      reward_type: vec![REWARD_GOAL_REACHED_BY_BALL, REWARD_SCOLD_INVALID_ACTION, REWARD_DISTANCE_TO_GOAL_IN_SIMULATION],
      target_type: 0,
      step_limit: 10,
      simulation_steps: (1000/80) * 100,
      physics_delta: 80,
      physics_substeps: 1,
      intermediate_simulation_frequency: 3,
      premature_end_after_steps_without_movement: (1000/80) * 3,
      premature_end_min_distance: 0.7,
      booster_strength: 0.25,
      use_cylinder_track: false,
      max_up_angle: vec![-90.0, 90.0],
      obs_sliding_window_size: 4,
      use_new_goalgen: false
    }
  }
}

pub fn is_freepoint_actionspace(action_type: u8) -> bool {
  match action_type {
    ACTION_TYPE_STATIC_WITH_BOOST | ACTION_TYPE_STATIC_WITH_EMPTY | ACTION_TYPE_STATIC => {false},
    _ => {true}
  }
}