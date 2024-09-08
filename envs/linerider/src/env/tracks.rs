use bevy::math::Vec3;
use crate::util::consts::*;
use super::LineRider3DEnv;

pub fn prepare_ramp_track(env: &mut LineRider3DEnv) {
  env.sim.set_max_width(50.0);
  env.sim.set_goal_position(Vec3::new(10.5, -3.5, 0.0));
  for _ in 0..5 {
    env.add_line_for_action(ACTION_DOWN_BOOST);
  }
  for _ in 0..2 {
    env.add_line_for_action(ACTION_UP);
  }
  env.add_line_for_action(ACTION_STRAIGHT_EMPTY);
  env.add_line_for_action(ACTION_STRAIGHT_EMPTY);
  env.add_line_for_action(ACTION_STRAIGHT);
}

pub fn make_freeroam_lines(env: &mut LineRider3DEnv) {
  env.sim.set_max_width(50.0);
  env.add_line_for_point((Vec3::new(0.0, 0.0, -1.1), TP_NORMAL));
  env.add_line_for_point((Vec3::new(-1.0, 0.0, -2.1), TP_NORMAL));
  env.add_line_for_point((Vec3::new(-1.0, -1.0, -3.1), TP_NORMAL));
  env.add_line_for_point((Vec3::new(-1.0, -1.0, -4.1), TP_NORMAL));
  env.add_line_for_point((Vec3::new(1.0, 0.0, -5.1), TP_NORMAL));
  env.add_line_for_point((Vec3::new(1.0, 0.0, -6.1), TP_NORMAL));
}

pub fn prepare_collision_track(env: &mut LineRider3DEnv) {
  env.sim.set_max_width(50.0);
  env.add_line_for_action(ACTION_DOWN);
  env.add_line_for_action(ACTION_LEFT);
  env.add_line_for_action(ACTION_LEFT);
  env.add_line_for_action(ACTION_LEFT);
  env.add_line_for_action(ACTION_RIGHT);
  env.add_line_for_action(ACTION_STRAIGHT);
  env.add_line_for_action(ACTION_DOWN);
  env.add_line_for_action(ACTION_UP);
  // for _ in 0..4 {
  //   env.step(&ArrayBase::from(vec![ACTION_LEFT as f64]).into_dyn());
  // }
}

pub fn prepare_curvy_track(env: &mut LineRider3DEnv) {
  env.sim.set_max_width(50.0);
  env.sim.set_goal_position(Vec3::new(1.0, -7.5, 1.0));
  env.add_line_for_action(ACTION_DOWN);
  env.add_line_for_action(ACTION_LEFT);
  env.add_line_for_action(ACTION_STRAIGHT_DOWN_EMPTY);
  env.add_line_for_action(ACTION_LEFT);
  env.add_line_for_action(ACTION_STRAIGHT_DOWN_EMPTY);
  env.add_line_for_action(ACTION_LEFT);
  env.add_line_for_action(ACTION_DOWN);
  env.add_line_for_action(ACTION_LEFT);
  env.add_line_for_action(ACTION_STRAIGHT_DOWN_EMPTY);
  env.add_line_for_action(ACTION_RIGHT);
  env.add_line_for_action(ACTION_DOWN);
  env.add_line_for_action(ACTION_RIGHT);
  env.add_line_for_action(ACTION_DOWN);
  env.add_line_for_action(ACTION_RIGHT);
  env.add_line_for_action(ACTION_DOWN);
  env.add_line_for_action(ACTION_RIGHT);
}


pub fn prepare_bezier_jump(env: &mut LineRider3DEnv) {
  env.sim.set_max_width(50.0);
  env.sim.set_goal_position(Vec3::new(10.0, 3.0, 0.0));
  env.add_line_for_point((Vec3::new(1.0, -1.0, 0.0), TP_NORMAL));
  env.add_line_for_point((Vec3::new(2.0, -1.5, 0.0), TP_NORMAL));
  env.add_line_for_point((Vec3::new(3.0, -2.0, 0.0), TP_NORMAL));
  env.add_line_for_point((Vec3::new(4., -1.75, 0.0), TP_ACCELERATE));
  env.add_line_for_point((Vec3::new(5.0, -1.5, 0.0), TP_ACCELERATE));
  env.add_line_for_point((Vec3::new(6.0, -1.0, 0.0), TP_ACCELERATE));
  env.add_line_for_point((Vec3::new(7.0, -0.5, 0.0), TP_ACCELERATE));
  env.add_line_for_point((Vec3::new(7.5, 0., 0.0), TP_ACCELERATE));
  // env.add_line_for_point((Vec3::new(9.25, 2.25, 0.0), TP_ACCELERATE));
}

pub fn prepare_all_types(env: &mut LineRider3DEnv) {
  env.sim.set_max_width(50.0);
  env.sim.set_goal_position(Vec3::new(7.0, -1.5, -3.0));
  env.add_line_for_action(ACTION_DOWN);
  env.add_line_for_action(ACTION_STRAIGHT);
  env.add_line_for_action(ACTION_UP);
  env.add_line_for_action(ACTION_STRAIGHT_DOWN);
  env.add_line_for_action(ACTION_LEFT);
  env.add_line_for_action(ACTION_STRAIGHT);
  env.add_line_for_action(ACTION_RIGHT);
  env.add_line_for_action(ACTION_STRAIGHT);
}

pub fn prepare_acc_jump(env: &mut LineRider3DEnv) {
  env.sim.set_max_width(50.0);
  env.add_line_for_action(ACTION_STRAIGHT);
  env.add_line_for_action(ACTION_STRAIGHT);
  env.add_line_for_action(ACTION_UP_BOOST);
  env.add_line_for_action(ACTION_UP_BOOST);
  env.add_line_for_action(ACTION_UP_BOOST);
  env.add_line_for_action(ACTION_UP_BOOST);
  env.add_line_for_action(ACTION_STRAIGHT_EMPTY);
  env.add_line_for_action(ACTION_STRAIGHT_EMPTY);
  env.add_line_for_action(ACTION_STRAIGHT);
  let mut new_goal = env.lines[env.lines.len()-1].0;
  new_goal.y += 0.5;
  new_goal.x += 0.8;
  env.sim.set_goal_position(new_goal);
}