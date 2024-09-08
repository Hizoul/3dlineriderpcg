pub mod range;
pub mod consts;
pub mod track;
#[cfg(feature = "libbuild")]
pub mod pylib;

use bevy::{math::Quat, prelude::Vec3};
use bevy_rapier3d::prelude::Velocity;
use nalgebra::Point3;

pub fn vel_to_f32(velocity: &Velocity) -> f32 {
  velocity.linvel.x.abs() + velocity.linvel.y.abs() + velocity.linvel.z.abs()
}

pub fn degree_to_radians(degree: f32) -> f32 {
  degree * std::f32::consts::PI / 180.0
}
pub fn radians_to_degree(radians: f32) -> f32 {
  radians * 180.0 / std::f32::consts::PI 
}

pub fn v_to_p(vec: &Vec3) -> Point3<f32> {
  Point3::new(vec.x, vec.y, vec.z)
}

pub fn middle_of_two_points(p1: &Vec3, p2: &Vec3) -> Vec3 {
  Vec3::new(
    (p1.x+p2.x) / 2.0,
    (p1.y+p2.y) / 2.0,
    (p1.z+p2.z) / 2.0,
  )
}

pub fn calculate_euler_angles(p1: Vec3, p2: Vec3) -> (f32, f32, f32) {
  let delta_x = p2.x - p1.x;
  let delta_y = p2.y - p1.y;
  let delta_z = p2.z - p1.z;
  let phi = delta_y.atan2(delta_x);
  let theta = (-delta_z / ((delta_x * delta_x + delta_y * delta_y).sqrt())).atan2(1.0);
  let initial_direction = Vec3::new(1.0, 0.0, 0.0);
  let rotation_axis = Vec3::new(0.0, 1.0, 0.0);
  let rotation_quat = Quat::from_axis_angle(rotation_axis, phi);
  let rotated_direction = rotation_quat.mul_vec3(initial_direction);
  let psi = rotated_direction.angle_between(initial_direction);
  (phi.to_degrees(), theta.to_degrees(), psi.to_degrees())
}
