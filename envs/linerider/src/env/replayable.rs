use crate::simulator::config::LineRiderConfig;
use bevy::prelude::Mut;
use super::LineRider3DEnv;
use rusty_gym::ReplayableGymEnvironment;
use std::collections::HashMap;

impl ReplayableGymEnvironment for LineRider3DEnv {
  fn get_used_seed(&mut self) -> u64 {self.used_seed}
  fn finalize(&mut self, _: &str, _: &str) {

  }
  fn get_config(&mut self) -> HashMap<String, String> {
    let mut env_conf = HashMap::new();
    env_conf.insert("name".to_owned(), self.get_name());
    env_conf.insert("max_width".to_owned(), self.sim.config.max_width.to_string());
    env_conf.insert("max_height".to_owned(), self.sim.config.max_height.to_string());
    env_conf.insert("rider_shape".to_owned(), self.sim.config.rider_shape.to_string());
    env_conf.insert("rider_mass".to_owned(), self.sim.config.rider_mass.to_string());
    env_conf.insert("rider_density".to_owned(), self.sim.config.rider_density.to_string());
    env_conf.insert("rider_size".to_owned(), serde_json::to_string(&self.sim.config.rider_size).expect("rider_size is serializable"));
    env_conf.insert("track_width".to_owned(), self.sim.config.track_width.to_string());
    env_conf.insert("track_wall_height".to_owned(), self.sim.config.track_wall_height.to_string());
    env_conf.insert("track_wall_width".to_owned(), self.sim.config.track_wall_width.to_string());
    env_conf.insert("track_piece_length".to_owned(), self.sim.config.track_piece_length.to_string());
    env_conf.insert("starting_force_multiplier".to_owned(), self.sim.config.starting_force_multiplier.to_string());
    env_conf.insert("skip_collision_check_on_last_x_pieces".to_owned(), self.sim.config.skip_collision_check_on_last_x_pieces.to_string());
    env_conf.insert("bezier_resolution".to_owned(), self.sim.config.bezier_resolution.to_string());
    env_conf.insert("smooth_free_points".to_owned(), self.sim.config.smooth_free_points.to_string());
    env_conf.insert("goal_size".to_owned(), self.sim.config.goal_size.to_string());
    env_conf.insert("goal_position".to_owned(), serde_json::to_string(&self.sim.config.goal_position).expect("goal_position is serializable"));
    env_conf.insert("checkpoint_range".to_owned(), serde_json::to_string(&self.sim.config.checkpoint_range).expect("checkpoint_range is serializable"));
    env_conf.insert("observation_type".to_owned(), self.sim.config.observation_type.to_string());
    env_conf.insert("action_type".to_owned(), self.sim.config.action_type.to_string());
    env_conf.insert("reward_type".to_owned(), serde_json::to_string(&self.sim.config.reward_type).expect("Reward type can be serialized to JSON"));
    env_conf.insert("target_type".to_owned(), self.sim.config.target_type.to_string());
    env_conf.insert("step_limit".to_owned(), self.sim.config.step_limit.to_string());
    env_conf.insert("simulation_steps".to_owned(), self.sim.config.simulation_steps.to_string());
    env_conf.insert("physics_delta".to_owned(), self.sim.config.physics_delta.to_string());
    env_conf.insert("physics_substeps".to_owned(), self.sim.config.physics_substeps.to_string());
    env_conf.insert("intermediate_simulation_frequency".to_owned(), self.sim.config.intermediate_simulation_frequency.to_string());
    env_conf.insert("premature_end_after_steps_without_movement".to_owned(), self.sim.config.premature_end_after_steps_without_movement.to_string());
    env_conf.insert("premature_end_min_distance".to_owned(), self.sim.config.premature_end_min_distance.to_string());
    env_conf.insert("booster_strength".to_owned(), self.sim.config.booster_strength.to_string());
    env_conf.insert("use_cylinder_track".to_owned(), self.sim.config.use_cylinder_track.to_string());
    env_conf.insert("max_up_angle".to_owned(), serde_json::to_string(&self.sim.config.max_up_angle).expect("maxupangle jsonable"));
    env_conf.insert("obs_sliding_window_size".to_owned(), self.sim.config.obs_sliding_window_size.to_string());
    env_conf.insert("use_new_goalgen".to_owned(), self.sim.config.use_new_goalgen.to_string());
    env_conf
  }
  fn load_config(&mut self, config: &HashMap<String, String>) {
    if config.keys().len() != 0 {
      if let Some(max_width) = config.get("max_width") {
        self.sim.set_max_width(max_width.parse().expect("value 'max_width' can be parsed"));
      }
      if let Some(max_height) = config.get("max_height") {
        self.sim.config.max_height = max_height.parse().expect("value 'max_height' can be parsed");
      }
      if let Some(rider_shape) = config.get("rider_shape") {
        self.sim.config.rider_shape = rider_shape.parse().expect("value 'rider_shape' can be parsed");
      }
      if let Some(rider_mass) = config.get("rider_mass") {
        self.sim.config.rider_mass = rider_mass.parse().expect("value 'rider_mass' can be parsed");
      }
      if let Some(rider_density) = config.get("rider_density") {
        self.sim.config.rider_density = rider_density.parse().expect("value 'rider_density' can be parsed");
      }
      if let Some(rider_size) = config.get("rider_size") {
        self.sim.config.rider_size = serde_json::from_str(rider_size).expect("value 'rider_size' can be parsed");
      }
      if let Some(track_width) = config.get("track_width") {
        self.sim.config.track_width = track_width.parse().expect("value 'track_width' can be parsed");
      }
      if let Some(track_wall_height) = config.get("track_wall_height") {
        self.sim.config.track_wall_height = track_wall_height.parse().expect("value 'track_wall_height' can be parsed");
      }
      if let Some(track_wall_width) = config.get("track_wall_width") {
        self.sim.config.track_wall_width = track_wall_width.parse().expect("value 'track_wall_width' can be parsed");
      }
      if let Some(track_piece_length) = config.get("track_piece_length") {
        self.sim.config.track_piece_length = track_piece_length.parse().expect("value 'track_piece_length' can be parsed");
      }
      if let Some(starting_force_multiplier) = config.get("starting_force_multiplier") {
        self.sim.config.starting_force_multiplier = starting_force_multiplier.parse().expect("value 'starting_force_multiplier' can be parsed");
      }
      if let Some(skip_collision_check_on_last_x_pieces) = config.get("skip_collision_check_on_last_x_pieces") {
        self.sim.config.skip_collision_check_on_last_x_pieces = skip_collision_check_on_last_x_pieces.parse().expect("value 'skip_collision_check_on_last_x_pieces' can be parsed");
      }
      if let Some(bezier_resolution) = config.get("bezier_resolution") {
        self.sim.config.bezier_resolution = bezier_resolution.parse().expect("value 'bezier_resolution' can be parsed");
      }
      if let Some(smooth_free_points) = config.get("smooth_free_points") {
        self.sim.config.smooth_free_points = smooth_free_points.parse().expect("value 'smooth_free_points' can be parsed");
      }
      if let Some(goal_size) = config.get("goal_size") {
        self.sim.config.goal_size = goal_size.parse().expect("value 'goal_size' can be parsed");
      }
      if let Some(goal_position) = config.get("goal_position") {
        self.sim.config.goal_position = serde_json::from_str(goal_position).expect("value 'goal_position' can be parsed");
      }
      if let Some(checkpoint_range) = config.get("checkpoint_range") {
        self.sim.config.checkpoint_range = serde_json::from_str(checkpoint_range).expect("value 'checkpoint_range' can be parsed");
      }
      if let Some(observation_type) = config.get("observation_type") {
        self.sim.config.observation_type = observation_type.parse().expect("value 'observation_type' can be parsed");
        self.observation_space = LineRider3DEnv::get_observation_space(&self.sim.config);
      }
      if let Some(action_type) = config.get("action_type") {
        self.sim.config.action_type = action_type.parse().expect("value 'action_type' can be parsed");
        self.action_space = LineRider3DEnv::get_action_space(&self.sim.config);
      }
      if let Some(reward_type) = config.get("reward_type") {
        self.sim.config.reward_type = serde_json::from_str(reward_type).expect("value 'reward_type' can be parsed");
      }
      if let Some(target_type) = config.get("target_type") {
        self.sim.config.target_type = target_type.parse().expect("value 'target_type' can be parsed");
      }
      if let Some(step_limit) = config.get("step_limit") { // Needs to be after observation_type restoration because history is dependent on step_limit
        self.sim.config.step_limit = step_limit.parse().expect("value 'step_limit' can be parsed");
        self.observation_space = LineRider3DEnv::get_observation_space(&self.sim.config);
      }
      if let Some(simulation_steps) = config.get("simulation_steps") {
        self.sim.config.simulation_steps = simulation_steps.parse().expect("value 'simulation_steps' can be parsed");
      }
      if let Some(physics_delta) = config.get("physics_delta") {
        self.sim.config.physics_delta = physics_delta.parse().expect("value 'physics_delta' can be parsed");
      }
      if let Some(physics_substeps) = config.get("physics_substeps") {
        self.sim.config.physics_substeps = physics_substeps.parse().expect("value 'physics_substeps' can be parsed");
      }
      if let Some(intermediate_simulation_frequency) = config.get("intermediate_simulation_frequency") {
        self.sim.config.intermediate_simulation_frequency = intermediate_simulation_frequency.parse().expect("value 'intermediate_simulation_frequency' can be parsed");
      }
      if let Some(max_piece_length) = config.get("max_piece_length") {
        self.sim.config.max_piece_length = max_piece_length.parse().expect("value 'max_piece_length' can be parsed");
      }
      if let Some(premature_end_after_steps_without_movement) = config.get("premature_end_after_steps_without_movement") {
        self.sim.config.premature_end_after_steps_without_movement = premature_end_after_steps_without_movement.parse().expect("value 'premature_end_after_steps_without_movement' can be parsed");
      }
      if let Some(premature_end_min_distance) = config.get("premature_end_min_distance") {
        self.sim.config.premature_end_min_distance = premature_end_min_distance.parse().expect("value 'premature_end_min_distance' can be parsed");
      }
      if let Some(booster_strength) = config.get("booster_strength") {
        self.sim.config.booster_strength = booster_strength.parse().expect("value 'booster_strength' can be parsed");
      }
      if let Some(use_cylinder_track) = config.get("use_cylinder_track") {
        self.sim.config.use_cylinder_track = use_cylinder_track.parse().expect("value 'use_cylinder_track' can be parsed");
      }      
      if let Some(max_up_angle) = config.get("max_up_angle") {
        self.sim.config.max_up_angle = serde_json::from_str(max_up_angle).expect("value 'max_up_angle' can be parsed");
      }
      if let Some(obs_sliding_window_size) = config.get("obs_sliding_window_size") {
        self.sim.config.obs_sliding_window_size = obs_sliding_window_size.parse().expect("value 'obs_sliding_window_size' can be parsed");
      }
      if let Some(use_new_goalgen) = config.get("use_new_goalgen") {
        self.sim.config.use_new_goalgen = use_new_goalgen.parse().expect("value 'use_new_goalgen' can be parsed");
      }
      if !self.sim.with_ui {
        self.sim.set_physics_delta(self.sim.config.physics_delta, self.sim.config.physics_substeps);
      } 
      {
        let mut app_config: Mut<LineRiderConfig> = self.sim.app.world.resource_mut();
        app_config.copy_from(&self.sim.config);
      }
    }
  }
  fn get_name(&self) -> String {"LineRider3D-Env-v0".to_owned()}
}
