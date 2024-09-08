use bevy::{prelude::Vec3, render::mesh::shape::Box};
use nalgebra::Scalar;
use serde::{Serialize, Deserialize};
use std::cmp::PartialOrd;

#[derive(Serialize, Deserialize)]
pub struct Range2D<X: Scalar + PartialOrd> {
  x_min: X,
  x_max: X,
  y_min: X,
  y_max: X
}

impl<X: Scalar + PartialOrd> Range2D<X> {
  pub fn new(x_min: X, x_max: X, y_min: X, y_max: X) -> Range2D<X> {
    Range2D {
      x_min, x_max, y_min, y_max
    }
  }
  pub fn is_in_x(&self, x_val: X) -> bool {
    self.x_min < x_val && x_val < self.x_max
  }
  pub fn is_in_y(&self, y_val: X) -> bool {
    self.y_min < y_val && y_val < self.y_max
  }
  pub fn is_in_range(&self, x_val: X, y_val: X) -> bool {
    self.is_in_x(x_val) && self.is_in_y(y_val)
  }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Range3D<X: Scalar + PartialOrd> {
  pub x_min: X,
  pub x_max: X,
  pub y_min: X,
  pub y_max: X,
  pub z_min: X,
  pub z_max: X
}

impl<X: Scalar + PartialOrd> Range3D<X> {
  pub fn new(x_min: X, x_max: X, y_min: X, y_max: X, z_min: X, z_max: X) -> Range3D<X> {
    Range3D {
      x_min, x_max, y_min, y_max, z_min, z_max
    }
  }
  pub fn is_in_x(&self, x_val: X) -> bool {
    self.x_min < x_val && x_val < self.x_max
  }
  pub fn is_in_y(&self, y_val: X) -> bool {
    self.y_min < y_val && y_val < self.y_max
  }
  pub fn is_in_z(&self, z_val: X) -> bool {
    self.z_min < z_val && z_val < self.z_max
  }
  pub fn is_in_range(&self, x_val: X, y_val: X, z_val: X) -> bool {
    self.is_in_x(x_val) && self.is_in_y(y_val) && self.is_in_z(z_val)
  }
}
impl Range3D<f32> {
  pub fn vec3_in_range(&self, pos: &Vec3) -> bool {
    self.is_in_range(pos.x, pos.y, pos.z)
  }
  pub fn to_box(&self) -> Box {
    Box {
      max_x: self.x_max,
      min_x: self.x_min,
      max_y: self.y_max,
      min_y: self.y_min,
      max_z: self.z_max,
      min_z: self.z_min,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_is_in_range_2d() {
    let my_range = Range2D::new(0.0, 32.0, -32.0, 0.0);
    assert_eq!(my_range.is_in_x(5.0), true);
    assert_eq!(my_range.is_in_x(31.0), true);
    assert_eq!(my_range.is_in_x(-0.5), false);
    assert_eq!(my_range.is_in_x(33.0), false);

    assert_eq!(my_range.is_in_y(-5.0), true);
    assert_eq!(my_range.is_in_y(-31.0), true);
    assert_eq!(my_range.is_in_y(0.5), false);
    assert_eq!(my_range.is_in_y(-33.0), false);
  }
  #[test]
  fn test_is_in_range_3d() {
    let my_range = Range3D::new(0.0, 32.0, -32.0, 0.0, 10.0, 20.0);
    assert_eq!(my_range.is_in_x(5.0), true);
    assert_eq!(my_range.is_in_x(31.0), true);
    assert_eq!(my_range.is_in_x(-0.5), false);
    assert_eq!(my_range.is_in_x(33.0), false);

    assert_eq!(my_range.is_in_y(-5.0), true);
    assert_eq!(my_range.is_in_y(-31.0), true);
    assert_eq!(my_range.is_in_y(0.5), false);
    assert_eq!(my_range.is_in_y(-33.0), false);

    assert_eq!(my_range.is_in_z(11.0), true);
    assert_eq!(my_range.is_in_z(19.0), true);
    assert_eq!(my_range.is_in_z(9.0), false);
    assert_eq!(my_range.is_in_z(21.0), false);
  }
}