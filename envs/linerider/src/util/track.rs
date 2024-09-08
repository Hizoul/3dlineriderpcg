use bevy::{prelude::{Vec3, Mesh, Quat}, render::{render_resource::PrimitiveTopology, mesh::Indices}};
use bevy_rapier3d::{prelude::Collider, parry::query::contact};
use nalgebra::{IsometryMatrix3, Vector3, Isometry3};
use crate::{util::consts::*, simulator::{LineRiderConfig, TrackPoint}};
use nalgebra as na;
use super::range::Range3D;

pub fn get_change_vector_for_movement_in_direction(direction: i64, action_val: i64, track_piece_length: f32) -> (Vec3, i64) {
  let mut change_by = Vec3::ZERO;
  let mut new_direction = direction;
  let mut change_index = match direction {
    DIRECTION_FORWARD | DIRECTION_BACK => {0},
    _ => {2}
  };
  let mut is_add = match direction {
    DIRECTION_FORWARD | DIRECTION_RIGHT => {true},
    _ => {false}
  };
  if is_add {change_by[change_index] += track_piece_length} else {change_by[change_index] -= track_piece_length}    
  match action_val {
    ACTION_LEFT | ACTION_RIGHT => {
      is_add = match direction {
        DIRECTION_RIGHT | DIRECTION_BACK => {true},
        _ => {false}
      };
      if action_val == ACTION_RIGHT {
        is_add = !is_add;
      }
      change_index = match direction {
        DIRECTION_RIGHT | DIRECTION_LEFT => {0},
        _ => {2}
      };
      if is_add {change_by[change_index] += track_piece_length} else {change_by[change_index] -= track_piece_length}
      new_direction = match direction {
        DIRECTION_FORWARD => {if action_val == ACTION_LEFT {DIRECTION_LEFT} else {DIRECTION_RIGHT}},
        DIRECTION_RIGHT => {if action_val == ACTION_LEFT {DIRECTION_FORWARD} else {DIRECTION_BACK}},
        DIRECTION_LEFT => {if action_val == ACTION_LEFT {DIRECTION_BACK} else {DIRECTION_FORWARD}},
        _ => {if action_val == ACTION_LEFT {DIRECTION_RIGHT} else {DIRECTION_LEFT}}
      };
    },
    ACTION_UP | ACTION_DOWN | ACTION_STRAIGHT_DOWN => {
      if action_val == ACTION_UP {
        change_by.y += track_piece_length * 0.5;
      } else {
        change_by.y -= track_piece_length;
      }
      if action_val == ACTION_STRAIGHT_DOWN {
        change_by.x = 0.0;
        change_by.z = 0.0;
      }
    },
    _ => {}
  }
  (change_by, new_direction)
}


pub fn get_direction(prev_direction: i64, prev_point: Vec3, current_point: Vec3) -> i64 {
  let middle_point = current_point - prev_point;
  match prev_direction {
    DIRECTION_FORWARD | DIRECTION_BACK => {
      if middle_point.z < 0.0 {
        DIRECTION_LEFT
      } else if middle_point.z > 0.0 {
        DIRECTION_RIGHT
      } else {
        prev_direction
      }
    },
    DIRECTION_LEFT | DIRECTION_RIGHT => {
      if middle_point.x > 0.0 {
        DIRECTION_FORWARD
      } else if middle_point.x < 0.0 {
        DIRECTION_BACK
      } else {
        prev_direction
      }
    }
    _ => {prev_direction}
  }
}
pub fn get_action(current_direction: i64, prev_point: Vec3, current_point: Vec3) -> i64 {
  let middle_point = current_point - prev_point;
  match current_direction {
    DIRECTION_FORWARD | DIRECTION_BACK => {
      if middle_point.y != 0.0  {
        if middle_point.y > 0.0 {ACTION_UP} else {if middle_point.x == 0.0 {ACTION_STRAIGHT_DOWN} else {ACTION_DOWN}}
      } else if middle_point.z != 0.0 {
        if middle_point.z > 0.0 {
          if current_direction == DIRECTION_FORWARD {ACTION_RIGHT} else {ACTION_LEFT}
        } else {
          if current_direction == DIRECTION_FORWARD {ACTION_LEFT} else {ACTION_RIGHT}
        }
      } else {
        ACTION_STRAIGHT
      }
    },
    DIRECTION_LEFT | DIRECTION_RIGHT => {
      if middle_point.y != 0.0  {
        if middle_point.y > 0.0 {ACTION_UP} else {if middle_point.z == 0.0 {ACTION_STRAIGHT_DOWN} else {ACTION_DOWN}}
      } else if middle_point.x != 0.0 {
        if middle_point.x > 0.0 {
          if current_direction == DIRECTION_LEFT {ACTION_RIGHT} else {ACTION_LEFT}
        } else {
          if current_direction == DIRECTION_LEFT {ACTION_LEFT} else {ACTION_RIGHT}}
      } else {
        ACTION_STRAIGHT
      }
    }
    _ => {ACTION_STRAIGHT}
  }
}


fn make_bezier_middle_point(prev_point: Vec3, current_point: Vec3, direction: i64) -> Vec3 {
  let mut middle_point = current_point - prev_point;
  if middle_point.y < 0.0 {
    let change_index = if direction == DIRECTION_FORWARD || direction == DIRECTION_BACK {0} else {2};
    middle_point[change_index] = middle_point[change_index] * 1.0;
  } else if middle_point.y > 0.0 {
    let change_index = if direction == DIRECTION_FORWARD || direction == DIRECTION_BACK {0} else {2};
    middle_point[change_index] = middle_point[change_index] * 0.4;
    middle_point.y = middle_point.y * 0.0;
  } else if middle_point.z != 0.0 {
    let change_index = if direction == DIRECTION_FORWARD || direction == DIRECTION_BACK {2} else {0};
    middle_point[change_index] = 0.0;
  }
  middle_point
}

pub fn quadratic_bezier(p0: Vec3, p1: Vec3, p2: Vec3, steps: usize) -> Vec<Vec3> {
  let mut points = Vec::with_capacity(steps);
  let step_size = 1.0 / steps as f32;
  let mut t = 0.0;
  for _ in 0..steps+1 {
    let q0 = p0.lerp(p1, t);
    let q1 = p1.lerp(p2, t);
    let new_point = q0.lerp(q1, t);
    points.push(new_point);
    t += step_size;
  }
  points
}

pub fn get_all_points(prev_real_point: Vec3, current_real_point: Vec3, prev_direction: i64, current_direction: i64, current_action: i64, config: &LineRiderConfig) -> Vec<Vec<Vec3>> {
  let wall_height = config.track_wall_height;
  let wall_width = config.track_wall_width;
  let track_width = config.track_width;
  let mut all_points: Vec<Vec<Vec3>> = Vec::with_capacity(10);
  for point_type in 0..5 {// 0: regular, 1: left side, 2: right side, 3: left wall, 4: right wall
    let mut prev = prev_real_point.clone();
    let mut cur = current_real_point.clone();
    match current_direction {
      DIRECTION_RIGHT => {
        match point_type {
          1 => {
            if current_action == ACTION_RIGHT {
              prev.z -= track_width;
            } else if current_action == ACTION_LEFT {
              prev.z += track_width;
            } else {
              prev.x += track_width;
            }
            cur.x += track_width;
          },
          2 => {
            if current_action == ACTION_RIGHT {
              prev.z += track_width;
            } else if current_action == ACTION_LEFT {
              prev.z -= track_width;
            }  else {
              prev.x -= track_width;
            }
            cur.x -= track_width;
          },
          3 => {
            prev.y += wall_height;
            cur.y += wall_height;
            if current_action == ACTION_RIGHT {
              prev.z -= track_width + wall_width;
            } else if current_action == ACTION_LEFT {
              prev.z += track_width + wall_width;
            }  else {
              prev.x += track_width + wall_width;
            }
            cur.x += track_width + wall_width;
          },
          4 => {
            prev.y += wall_height;
            cur.y += wall_height;
            if current_action == ACTION_RIGHT {
              prev.z += track_width + wall_width;
            } else if current_action == ACTION_LEFT {
              prev.z -= track_width + wall_width;
            }  else {
              prev.x -= track_width + wall_width;
            }
            cur.x -= track_width + wall_width;
          },
          _ => {}
        }
      },
      DIRECTION_LEFT => {
        match point_type {
          1 => {
            if current_action == ACTION_LEFT {
              prev.z -= track_width;
            } else if current_action == ACTION_RIGHT {
              prev.z += track_width;
            }  else {
              prev.x -= track_width;
            }
            cur.x -= track_width;
          },
          2 => {
            if current_action == ACTION_LEFT {
              prev.z += track_width;
            } else if current_action == ACTION_RIGHT {
              prev.z -= track_width;
            }  else {
              prev.x += track_width;
            }
            cur.x += track_width;
          },
          3 => {
            prev.y += wall_height;
            cur.y += wall_height;
            if current_action == ACTION_LEFT {
              prev.z -= track_width + wall_width;
            } else if current_action == ACTION_RIGHT {
              prev.z += track_width + wall_width;
            }  else {
              prev.x -= track_width + wall_width;
            }
            cur.x -= track_width + wall_width;
          },
          4 => {
            prev.y += wall_height;
            cur.y += wall_height;
            if current_action == ACTION_LEFT {
              prev.z += track_width + wall_width;
            } else if current_action == ACTION_RIGHT {
              prev.z -= track_width + wall_width;
            }  else {
              prev.x += track_width + wall_width;
            }
            cur.x += track_width + wall_width;
          },
          _ => {}
        }
      },
      DIRECTION_BACK => {
        match point_type {
          1 => {
            if current_action == ACTION_LEFT {
              prev.x -= track_width;
            } else if current_action == ACTION_RIGHT {
              prev.x += track_width;
            } else {
              prev.z += track_width;
            }
            cur.z += track_width;
          },
          2 => {
            if current_action == ACTION_LEFT {
              prev.x += track_width;
            } else if current_action == ACTION_RIGHT {
              prev.x -= track_width;
            }  else {
              prev.z -= track_width;
            }
            cur.z -= track_width;
          },
          3 => {
            prev.y += wall_height;
            cur.y += wall_height;
            if current_action == ACTION_LEFT {
              prev.x -= track_width + wall_width;
            } else if current_action == ACTION_RIGHT {
              prev.x += track_width + wall_width;
            }  else {
              prev.z += track_width + wall_width;
            }
            cur.z += track_width + wall_width;
          },
          4 => {
            prev.y += wall_height;
            cur.y += wall_height;
            if current_action == ACTION_LEFT {
              prev.x += track_width + wall_width;
            } else if current_action == ACTION_RIGHT {
              prev.x -= track_width + wall_width;
            }  else {
              prev.z -= track_width + wall_width;
            }
            cur.z -= track_width + wall_width;
          },
          _ => {}
        }
      },
      _ => {
        match point_type {
          1 => {
            if current_action == ACTION_RIGHT {
              prev.x -= track_width;
            } else if current_action == ACTION_LEFT {
              prev.x += track_width;
            }  else {
              prev.z -= track_width;
            }
            cur.z -= track_width
          },
          2 => {
            if current_action == ACTION_RIGHT {
              prev.x += track_width;
            } else if current_action == ACTION_LEFT {
              prev.x -= track_width;
            }  else {
              prev.z += track_width;
            }
            cur.z += track_width
          },
          3 => {
            prev.y += wall_height;
            cur.y += wall_height;
            if current_action == ACTION_RIGHT {
              prev.x -= track_width + wall_width;
            } else if current_action == ACTION_LEFT {
              prev.x += track_width + wall_width;
            }  else {
              prev.z -= track_width + wall_width;
            }
            cur.z -= track_width + wall_width;
          },
          4 => {
            prev.y += wall_height;
            cur.y += wall_height;
            if current_action == ACTION_RIGHT {
              prev.x += track_width + wall_width;
            } else if current_action == ACTION_LEFT {
              prev.x -= track_width + wall_width;
            }  else {
              prev.z += track_width + wall_width;
            }
            cur.z += track_width + wall_width;
          },
          _ => {}
        }
      }
    }
    let is_curve = match current_action {
      ACTION_LEFT | ACTION_RIGHT => {true},
      _ => {false}
    };
    let change_middle_by = make_bezier_middle_point(prev, cur, prev_direction);
    let middle_point = prev + change_middle_by;
    let sub_points = quadratic_bezier(prev, middle_point, cur, if is_curve {config.bezier_resolution} else {6});
    all_points.push(sub_points);  
  }
  all_points
}


pub const MESH_INDICES: [u32; 24] = [0, 1, 2, 1, 4, 2, 0, 5, 1, 0, 3, 5, 2, 4, 6, 4, 7, 6, 3, 9, 5, 3, 8, 9];
pub const MESH_INDICES_COLLIDER: [[u32; 3]; 8] = [[0, 1, 2], [1, 4, 2], [0, 5, 1], [0, 3, 5], [2, 4, 6], [4, 7, 6], [3, 9, 5], [3, 8, 9]];
pub const MESH_INDICES_TRACK_COL: [[u32; 3]; 4] = [[0, 1, 2], [1, 4, 2], [0, 5, 1], [0, 3, 5]];
pub const MESH_INDICES_LEFT_WALL: [[u32; 3]; 2] = [[2, 4, 6], [4, 7, 6]];
pub const MESH_INDICES_RIGHT_WALL: [[u32; 3]; 2] = [[3, 9, 5], [3, 8, 9]];

pub fn get_mesh_points(all_points: &[Vec<Vec3>], u: usize) -> Vec<Vec3> {
  vec![
    all_points[0][u-1],   // 0 prev_point
    all_points[0][u],     // 1 current_point
    all_points[1][u-1],   // 2 prev_left
    all_points[2][u-1],   // 3 prev_right
    all_points[1][u],     // 4 cur_left
    all_points[2][u],     // 5 cur_right
    all_points[3][u-1],   // 6 prev_left_wall
    all_points[3][u],     // 7 cur_left_wall
    all_points[4][u-1],   // 8 prev_right_wall
    all_points[4][u]      // 9 cur_right_wall
  ]
}

pub fn make_track_mesh(all_points: &[Vec<Vec3>], u: usize) -> (Mesh, Vec<Vec3>) {
  let mesh_points = get_mesh_points(all_points, u);
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  let collected: Vec<[f32; 3]> = mesh_points.iter().map(|p| [p.x, p.y, p.z]).collect();
  let normals: Vec<[f32; 3]> = mesh_points.iter().map(|_p| [1.0, 1.0, 1.0]).collect();
  let mut uvs: Vec<[f32; 2]> = Vec::new(); //mesh_points.iter().map(|_p| [0.0, 0.0]).collect();
  for i in 0..mesh_points.len() {
    let to_push = if i % 2 == 0 {
      [1.0, 1.0]
    } else {
      [0.0, 0.0]
    };
    uvs.push(to_push);
  }
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, collected);
  mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
  mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
  mesh.set_indices(Some(Indices::U32(MESH_INDICES.to_vec())));
  (mesh, mesh_points)
}

pub fn aabbs_intersect(aabb: &(Vec3, Vec3), prev_aabb: &(Vec3, Vec3)) -> bool {
  aabb.0.x < prev_aabb.1.x &&
  aabb.1.x > prev_aabb.0.x &&
  aabb.0.y < prev_aabb.1.y &&
  aabb.1.y > prev_aabb.0.y &&
  aabb.0.z < prev_aabb.1.z &&
  aabb.1.z > prev_aabb.0.z
}

pub fn check_collision_with_colliders(colliders: &[Collider], new_collider: &Collider, amount_of_colliders_to_skip: usize, prediction: Option<f32>) -> bool {
  let p1 = Isometry3::translation(0.0, 0.0, 0.0);
  if colliders.len() > amount_of_colliders_to_skip {
    let v2 = colliders.len()-amount_of_colliders_to_skip;
    for collider in &colliders[0..v2] {
      let cont = contact(&p1, &*collider.raw, &p1, &*new_collider.raw, prediction.unwrap_or(0.09)).expect("");
      if let Some(contact) = cont {
        if contact.dist > 0.0 {
          return true;
        }
      }
    }
  }
  false  
}

pub fn check_collision(aabbs: &[(Vec3, Vec3)], aabb_tuple: (Vec3, Vec3)) -> bool {
  for prev_aabb_tuple in aabbs {
    if aabbs_intersect(prev_aabb_tuple, &aabb_tuple) {
      return true;
    }
  }
  false
}
pub type PrevPoints = [Vec3; 4];
pub fn get_free_mesh_points(prev_point: Vec3, current_point: Vec3, prev_points: &PrevPoints, config: &LineRiderConfig) -> (Vec<Vec<Vec3>>, Vec3) {
  let track_width = config.track_width;
  let wall_height = config.track_wall_height;
  let mut prev_left = prev_points[0];
  let mut prev_right = prev_points[1];
  let mut prev_left_wall = prev_points[2];
  let mut prev_right_wall = prev_points[3];
  let diff1 = current_point - prev_point;
  let iso = IsometryMatrix3::look_at_rh(&prev_point.into(), &current_point.into(), &Vector3::y());
  let mut rotation_angle = iso.rotation.angle();
  if diff1.x > 0.0 {
    rotation_angle *= -1.0;
  }
  if rotation_angle.is_nan() {
    rotation_angle = 0.0;
  }
  let move_track_width = Vec3::X * track_width;
  let move_wall_x = Vec3::X * (track_width + wall_height);
  let move_wall_y = Vec3::Y * wall_height;
  let rotate_by = Quat::from_axis_angle(Vec3::Y, rotation_angle);
  if prev_left == Vec3::ZERO {
    prev_left = prev_point - (rotate_by * move_track_width);
    prev_left_wall = prev_point - (rotate_by * move_wall_x) + move_wall_y;
    prev_right = prev_point + (rotate_by * move_track_width);
    prev_right_wall = prev_point + (rotate_by * move_wall_x) + move_wall_y;
  }
  let cur_left = current_point - (rotate_by * move_track_width);
  let cur_left_wall = current_point - (rotate_by * move_wall_x) + move_wall_y;
  let cur_right = current_point + (rotate_by * move_track_width);
  let cur_right_wall = current_point + (rotate_by * move_wall_x) + move_wall_y;
  (vec![
    vec![prev_point, current_point],
    vec![prev_left, cur_left],
    vec![prev_right, cur_right],
    vec![prev_left_wall, cur_left_wall],
    vec![prev_right_wall, cur_right_wall]
  ], diff1)
}

// ported from Three.js src/extras/curves/CatmullRomCurve3.js
pub fn nonuniform_catmull_rom(x0: f32, x1: f32, x2: f32, x3: f32, dt0: f32, dt1: f32, dt2: f32, weight: f32) -> f32 {
  let mut t1 = (x1 - x0) / dt0 - (x2 - x0) / (dt0 + dt1) + (x2-x1) / dt1;
  let mut t2 = (x2 - x1) / dt1 - (x3 - x1) / (dt1 + dt2) + (x3-x2) / dt2;
  t1 *= dt1;
  t2 *= dt1;
  let c0 = x0;
  let c1 = t1;
  let c2 = -3.0 * x0 + 3.0 * x1 - 2.0 * t1 - t2;
  let c3 = 2.0 * x0 - 2.0 * x1 + t1 + t2;
  t2 = weight * weight;
  let t3 = t2 * weight;
  let result = c0 + c1 * weight + c2 * t2 + c3 * t3;
  return result;
}
// ported from Three.js src/extras/curves/CatmullRomCurve3.js
pub fn catmull_rom(points: &[TrackPoint], divisions: usize) -> Vec<TrackPoint> {
  let mut new_points = Vec::with_capacity(divisions);
  for d in 0..divisions {
    let t = d as f32 / divisions as f32;

    let l = points.len();
    let p = (l as f32 - 1.0) * t;
    let mut int_point = p.floor() as usize;
    let mut weight = p - int_point as f32;

    if weight == 0.0 && int_point == l - 1 {
      int_point = l - 2;
      weight = 1.0;
    }

    let p0 = if int_point > 0 {
      points[(int_point-1) % l].0
    } else {
      points[0].0 - points[1].0 + points[0].0
    };
    let p3 = if int_point + 2 < l {
      points[(int_point+2) % l].0
    } else {
      points[l-1].0 - points[l - 2].0 + points[l-1].0
    };

    let p1 = points[int_point % l].0;
    let p2 = points[(int_point+1)%l].0;


    let pow_with = 0.25;

    let mut dt0 = p0.distance_squared(p1).powf(pow_with);
    let mut dt1 = p1.distance_squared(p2).powf(pow_with);
    let mut dt2 = p2.distance_squared(p3).powf(pow_with);

    if dt1 < 1e-4 {dt1 = 1.0}
    if dt0 < 1e-4 {dt0 = dt1}
    if dt2 < 1e-4 {dt2 = dt1}

    let px = nonuniform_catmull_rom(p0.x, p1.x, p2.x, p3.x, dt0, dt1, dt2, weight);
    let py = nonuniform_catmull_rom(p0.y, p1.y, p2.y, p3.y, dt0, dt1, dt2, weight);
    let pz = nonuniform_catmull_rom(p0.z, p1.z, p2.z, p3.z, dt0, dt1, dt2, weight);

    let new_point = Vec3::new(px, py, pz);
    new_points.push((new_point, points[(int_point+1)%l].1));
  }
  new_points
}


pub fn make_build_range(config: &LineRiderConfig, multiplier: f32, origin_opt: &Option<Vec3>) -> Range3D<f32> {
  let half_width = config.max_width * multiplier;
  if let Some(origin) = origin_opt {
    Range3D::new(
      origin.x-half_width, origin.x+half_width,
      origin.y-half_width, origin.y+half_width,
      origin.z-half_width, origin.z+half_width,
    )
  } else {
    Range3D::new(
      -half_width, half_width,
      -half_width, half_width,
      -half_width, half_width,
    )
  }
}

pub fn make_goal_range(goal_pos: &Vec3, config: &LineRiderConfig) -> Range3D<f32> {
  let width = config.goal_size;
  let height = config.goal_size;
  let depth = config.goal_size;
  Range3D::new(
    goal_pos.x - width, goal_pos.x + width,
    goal_pos.y - height, goal_pos.y + height,
    goal_pos.z - depth, goal_pos.z + depth
  )
}

pub fn make_goal_pos(goal_pos: &Range3D<f32>) -> Vec3 {
  let goal_x = (goal_pos.x_min + goal_pos.x_max) / 2.0;
  let goal_y = (goal_pos.y_min + goal_pos.y_max) / 2.0;
  let goal_z = (goal_pos.z_min + goal_pos.z_max) / 2.0;
  Vec3::new(goal_x, goal_y, goal_z)
}

pub fn devin(path: &[Vec3], radius: f32, segments: usize) -> (Vec<Vec3>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..path.len() {
        let p0 = path[i];
        let p1 = if i == path.len() - 1 {
            path[i]
        } else {
            path[i + 1]
        };

        let direction = (p1 - p0).normalize();
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        for j in 0..segments {
            let angle = (j as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let vertex = p0 + rotation * Vec3::new(x, 0.0, z);
            vertices.push(vertex);

            if i > 0 {
                let base_index = ((i - 1) * segments + j) as u32;
                indices.push(base_index);
                indices.push(base_index + segments as u32);
                indices.push(base_index + 1);

                indices.push(base_index + 1);
                indices.push(base_index + segments as u32);
                indices.push(base_index + segments as u32 + 1);
            }
        }
    }

    (vertices, indices)
}

pub fn generate_cylinder_vertices(start: na::Point3<f32>, end: na::Point3<f32>, radius: f32, segments: usize) -> (Vec<Vec3>, Vec<u32>) {
  let mut vertices = Vec::new();
  let mut indices = Vec::new();

  // Calculate the direction vector from start_point to end_point
  let direction = end - start;

  // Calculate the length of the pipe segment
  let length = direction.norm();

  // Calculate the rotation quaternion to align the pipe with the direction vector
  let rotation = na::UnitQuaternion::rotation_between(&na::Vector3::z_axis(), &direction).unwrap();

  // Generate vertices
  for i in 0..=segments {
      let theta = i as f32 / segments as f32 * 2.0 * std::f32::consts::PI;

      // Calculate position in local coordinates
      let local_position = na::Point3::new(radius * theta.cos(), radius * theta.sin(), 0.0);

      // Rotate and translate to world coordinates
      let position = start + rotation.transform_vector(&local_position.coords);
      vertices.push(Vec3::new(position.x, position.y, position.z));
  }

  // Generate indices
  for i in 0..segments {
      let i0 = i as u32;
      let i1 = (i + 1) as u32;

      // Add two triangles to form a quad
      indices.push(i0);
      indices.push(i1);
      indices.push((i0 + 1) % (segments as u32 + 1));
  
      indices.push(i1);
      indices.push((i1 + 1) % (segments as u32 + 1));
      indices.push((i0 + 1) % (segments as u32 + 1));
  }

  (vertices, indices)
}


pub fn generate_pipe_vertices(start: Vec3, end: Vec3, radius: f32, segments: usize, prev_end_vertices: Option<Vec<Vec3>>) -> (Vec<Vec3>, Vec<u32>, Vec<Vec3>) {
  let mut vertices = Vec::new();
  let mut indices = Vec::new();

  let direction = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
  let height = f32::sqrt(direction[0].powi(2) + direction[1].powi(2) + direction[2].powi(2));
  let axis = [0.0, 1.0, 0.0];

  let rotation_axis = [
      axis[1] * direction[2] - axis[2] * direction[1],
      axis[2] * direction[0] - axis[0] * direction[2],
      axis[0] * direction[1] - axis[1] * direction[0],
  ];

  let rotation_angle = direction[0].atan2(direction[2]);
  let mut end_vertices = Vec::new();
  // Generate vertices
  for i in 0..segments {
      let theta = i as f32 / segments as f32 * 2.0 * std::f32::consts::PI;
      let x = radius * theta.cos();
      let z = radius * theta.sin();
      // // Rotate the circle points to align with the direction vector
      let rotated_x: f32 = x * rotation_axis[0] + z * rotation_axis[2];
      let rotated_z = x * rotation_axis[2] - z * rotation_axis[0];
      let translated_x = rotated_x + start[0];
      let translated_y = start[1] + (i as f32) * height / (segments as f32);
      let translated_z = rotated_z + start[2];
      if let Some(prev_end) = &prev_end_vertices {
        vertices.push(prev_end[i]);
      } else {
        vertices.push(Vec3::new(translated_x, start[1] + (i as f32) * height / (segments as f32), translated_z));
      }
      
      let translated_x = rotated_x + end[0];
      let translated_y = end[1] + (i as f32) * height / (segments as f32);
      let translated_z = rotated_z + end[2];
        vertices.push(Vec3::new(translated_x, translated_y, translated_z));
      end_vertices.push(Vec3::new(translated_x, translated_y, translated_z));

      let index_offset = i as u32 * 2;
      indices.push(index_offset);
      indices.push(index_offset + 1);
      indices.push((index_offset + 2) % (segments as u32 * 2));

      indices.push(index_offset + 1);
      indices.push((index_offset + 3) % (segments as u32 * 2));
      indices.push((index_offset + 2) % (segments as u32 * 2));


      // // Translate the points to the start position
      // let translated_x = rotated_x + start[0];
      // let translated_y = start[1] + (i as f32) * height / (segments as f32);
      // let translated_z = rotated_z + start[2];

      // // Add the vertex
      // vertices.push(Vec3::new(translated_x, translated_y, translated_z));
  }

  // Generate indices
  // for i in 0..segments {
  //     let i1 = i;
  //     let i2 = (i + 1) % segments;
  //     let i3 = i + segments;
  //     let i4 = (i + 1) % segments + segments;

  //     // Bottom face
  //     indices.push(i1 as u32);
  //     indices.push(i2 as u32);
  //     indices.push(i3 as u32);

  //     // Top face
  //     indices.push(i4 as u32);
  //     indices.push(i3 as u32);
  //     indices.push(i2 as u32);

  //     // Side faces
  //     indices.push(i1 as u32);
  //     indices.push(i2 as u32);
  //     indices.push(i4 as u32);
  //     indices.push(i1 as u32);
  //     indices.push(i4 as u32);
  //     indices.push(i3 as u32);
  // }

  (vertices, indices, end_vertices)
}

#[cfg(test)]
mod tests {
  use super::*;
  use insta::assert_debug_snapshot;
  #[test]
  fn bezier_curve() {
    let prev_point = Vec3::new(0.0, 0.0, 0.0);
    let current_point = Vec3::new(1.0, -1.0, 0.0);
    let middle_point = Vec3::new(0.0, -1.0, 0.0);
    let curve = quadratic_bezier(prev_point, middle_point, current_point, 5);
    assert_debug_snapshot!("Curve", curve);
  }

  #[test]
  fn direction_inference() {
    let config = LineRiderConfig::default();
    let check_direction = |prev_direction: i64, action: i64, expected_direction: i64| {
      let prev_point = Vec3::ZERO;
      let (change_by, new_direction) = get_change_vector_for_movement_in_direction(prev_direction, action, config.track_piece_length);
      assert_eq!(new_direction, expected_direction);
      assert_eq!(new_direction, get_direction(prev_direction, prev_point, change_by));
      assert_eq!(action, get_action(prev_direction, prev_point, change_by));
    };
    check_direction(DIRECTION_FORWARD, ACTION_DOWN, DIRECTION_FORWARD);
    check_direction(DIRECTION_FORWARD, ACTION_UP, DIRECTION_FORWARD);
    check_direction(DIRECTION_FORWARD, ACTION_STRAIGHT, DIRECTION_FORWARD);
    check_direction(DIRECTION_FORWARD, ACTION_LEFT, DIRECTION_LEFT);
    check_direction(DIRECTION_FORWARD, ACTION_RIGHT, DIRECTION_RIGHT);

    check_direction(DIRECTION_LEFT, ACTION_DOWN, DIRECTION_LEFT);
    check_direction(DIRECTION_LEFT, ACTION_UP, DIRECTION_LEFT);
    check_direction(DIRECTION_LEFT, ACTION_STRAIGHT, DIRECTION_LEFT);
    check_direction(DIRECTION_LEFT, ACTION_LEFT, DIRECTION_BACK);
    check_direction(DIRECTION_LEFT, ACTION_RIGHT, DIRECTION_FORWARD);

    check_direction(DIRECTION_RIGHT, ACTION_DOWN, DIRECTION_RIGHT);
    check_direction(DIRECTION_RIGHT, ACTION_UP, DIRECTION_RIGHT);
    check_direction(DIRECTION_RIGHT, ACTION_STRAIGHT, DIRECTION_RIGHT);
    check_direction(DIRECTION_RIGHT, ACTION_LEFT, DIRECTION_FORWARD);
    check_direction(DIRECTION_RIGHT, ACTION_RIGHT, DIRECTION_BACK);

    check_direction(DIRECTION_BACK, ACTION_DOWN, DIRECTION_BACK);
    check_direction(DIRECTION_BACK, ACTION_UP, DIRECTION_BACK);
    check_direction(DIRECTION_BACK, ACTION_STRAIGHT, DIRECTION_BACK);
    check_direction(DIRECTION_BACK, ACTION_LEFT, DIRECTION_RIGHT);
    check_direction(DIRECTION_BACK, ACTION_RIGHT, DIRECTION_LEFT);
  }
}
