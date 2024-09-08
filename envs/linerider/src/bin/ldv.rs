// use linerider::{env::{*, tracks::*}, simulator::LineRiderConfig};

use bevy::prelude::Vec3;
use rusty_gym::{gym::GymEnvironment, Observation, ReplayableGymEnvironment};
use linerider::{*, simulator::*, env::{tracks::{prepare_acc_jump, prepare_curvy_track, prepare_bezier_jump}, *}, algo::heuristic::straight_line_heuristic_general, util::consts::*};
use ndarray::ArrayBase;
// use simulator::*;
// use util::{consts::*, track::*};
use bevy::prelude::Mut;

fn debug_main() {
  let mut env = linerider::env::LineRider3DEnv::new(linerider::simulator::LineRiderSim::default_with_ui(), None);
  env.sim.add_debug_cam();
  env.sim.set_max_width(100.0);
  env.sim.set_goal_position(Vec3::new(100.0, 100.0, 100.0));
  {
    env.sim.config.use_cylinder_track = true;
    // env.sim.config.smooth_free_points = true;
    env.sim.config.bezier_resolution = 4;
    let mut app_config: Mut<LineRiderConfig> = env.sim.app.world.resource_mut();
    app_config.copy_from(&env.sim.config);
  }
  env.sim.config.action_type = 5;
  // for i in 0..10 {
  //   env.add_line_for_point((Vec3::new(i as f32, -i as f32, i as f32 * 0.1), 1));
  // }
  // for i in 0..10 {
  //   // env.add_line_for_point((Vec3::new(10 as f32, -10 as f32, i as f32), 1));
  // }
  env.add_line_for_point((Vec3::new(1.0, -1.0, 0.0), 1));
  env.add_line_for_point((Vec3::new(2.0, -2.0, 1.0), 1));
  env.add_line_for_point((Vec3::new(3.0, -2.0, 2.0), 1));
  env.add_line_for_point((Vec3::new(2.0, -1.0, 0.0), 1));
  env.add_line_for_point((Vec3::new(3.0, -1.0, 0.0), 1));
  env.add_line_for_point((Vec3::new(4.0, -1.0, 0.0), 1));
  env.add_line_for_point((Vec3::new(3.0, -4.0, 5.0), 1));
  env.add_line_for_point((Vec3::new(4.0, -4.0, 5.0), 1));

  // env.sim.add_driver_cam();

  // prepare_bezier_jump(&mut env);
  env.add_lines_freeroam();
  // env.add_line_for_action(ACTION_STRAIGHT);
  // env.add_line_for_action(ACTION_STRAIGHT);
  // env.add_line_for_action(ACTION_LEFT);
  // env.add_line_for_action(ACTION_STRAIGHT);
  // env.add_line_for_action(ACTION_STRAIGHT);
  // env.add_line_for_action(ACTION_LEFT);
  // env.add_line_for_action(ACTION_RIGHT);
  // env.add_lines();

  env.sim.app.run();
}

fn test_checkpoint() {
  let mut sim: LineRiderSim = LineRiderSim::default_with_ui();
  sim.config.use_cylinder_track = true;
  sim.config.smooth_free_points = true;
  sim.config.bezier_resolution = 10;
  sim.config.action_type = ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE;
  sim.config.target_type = TARGET_RANDOM_WITH_CHECKPOINT_BELOW;
  let mut env: LineRider3DEnv = LineRider3DEnv::new(sim, None);
  env.sim.config.step_limit = 10;
  env.skip_simulation = true;
  env.use_seed(42);
  let config = env.get_config();
  let mut obs: Observation = env.reset();
  println!("OBS IS {}", obs);
  let mut current_episode = 0;
  let mut success_counter = 0;
  for _ in 0..10 {
    println!("ANSWER IS {}", straight_line_heuristic_general(&obs, &config, None));
    let res = env.step(&straight_line_heuristic_general(&obs, &config, None));
    obs = res.obs;
  }
  env.add_lines_freeroam();
  env.sim.app.run();
}

fn main() {
  // test_checkpoint();
  //linerider::replay::replay_viewer();
  //linerider::replay::single::replay_single_episode_cli();
  debug_main();
}