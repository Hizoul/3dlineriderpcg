use crate::{
  gym::{GymEnvironment, Step, Observation, Action},
  space::Space,
  util::rng::{UniRng, get_rng_for_type, uni_gen_range_f64}
};
use ndarray::{ArrayBase, Array1};


pub struct MountainCar {
  rng: UniRng,
  action_space: Space,
  observation_space: Space,
  gravity: f64,
  force: f64,
  min_position: f64,
  max_position: f64,
  max_speed: f64,
  goal_position: f64,
  goal_velocity: f64,
  position: f64,
  velocity: f64,
  used_seed: u64,
  step: u64,
  max_steps: u64,
  broader_start_position: bool
}

impl std::fmt::Debug for MountainCar {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("Point")
       .field("goal_position", &self.goal_position)
       .finish()
  }
}

impl MountainCar {
  pub fn new(seed: Option<u64>) -> MountainCar {
    let (rng, used_seed) = get_rng_for_type(&None, seed);
    let min_position = -1.2;
    let max_position = 0.6;
    let max_speed = 0.07;
    let mut env = MountainCar {
      used_seed,
      rng,
      action_space: Space::Discrete(3),
      observation_space: Space::BoxedWithoutRange(vec![2]),
      gravity: 0.0025,
      force: 0.001,
      min_position,
      max_position,
      max_speed,
      goal_position: 0.5,
      goal_velocity: 0.0,
      position: -0.6,
      velocity: 0.0,
      step: 0,
      max_steps: 500,
      broader_start_position: false
    };
    env.reset_state();
    env
  }
  pub fn reset_state(&mut self) {
    let (min, max) = if self.broader_start_position { (self.min_position+0.05, self.goal_position-0.1 ) } else { (-0.6, -0.4) };
    self.position = uni_gen_range_f64(&None, &mut self.rng, min, max);
    self.velocity = 0.0;
    self.step = 0;
  }

  pub fn make_obs(&self) -> Observation {
    ArrayBase::from(vec![self.position, self.velocity]).into_dyn()
  }

  pub fn _height(x: f64) -> f64 {
    (3.0 * x).sin() * 0.45 + 0.55
  }
}

impl Default for MountainCar {
  fn default() -> MountainCar {MountainCar::new(None)}
}

impl GymEnvironment for MountainCar {
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
    let action_value = action[0].round();
    let acceleration_direction = action_value - 1.0;

    self.velocity += acceleration_direction * self.force + (3.0 * self.position).cos() * (-self.gravity);
    self.velocity = self.velocity.clamp(-self.max_speed, self.max_speed);

    self.position += self.velocity;
    self.position = self.position.clamp(self.min_position, self.max_position);
    if self.position == self.min_position && self.velocity < 0.0 {
      self.velocity = 0.0
    }

    self.step += 1;
    let is_done = self.step >= self.max_steps || (self.position >= self.goal_position && self.velocity >= self.goal_velocity);
    // println!("Episode is done because limit {}, outside bounds {} tipped over {}", self.step >= self.step_limit, cart_is_outside_of_bounds, pole_is_tipped_over);
    let reward = -1.0;

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


#[cfg(feature = "eval")]
use crate::ReplayableGymEnvironment;
#[cfg(feature = "eval")]
use std::collections::HashMap;
#[cfg(feature = "eval")]
impl ReplayableGymEnvironment for MountainCar {
  fn get_used_seed(&mut self) -> u64 {self.used_seed}
  fn get_config(&mut self) -> HashMap<String, String> {
    let mut env_conf = HashMap::new();
    env_conf.insert("name".to_owned(), self.get_name());
    env_conf.insert("min_position".to_owned(), self.min_position.to_string());
    env_conf.insert("gravity".to_owned(), self.gravity.to_string());
    env_conf.insert("max_position".to_owned(), self.max_position.to_string());
    env_conf.insert("max_speed".to_owned(), self.max_speed.to_string());
    env_conf.insert("goal_position".to_owned(), self.goal_position.to_string());
    env_conf.insert("goal_velocity".to_owned(), self.goal_velocity.to_string());
    env_conf.insert("force".to_owned(), self.force.to_string());
    env_conf.insert("max_steps".to_owned(), self.max_steps.to_string());
    env_conf.insert("broader_start_position".to_owned(), self.broader_start_position.to_string());
    env_conf
  }
  fn load_config(&mut self, config: &HashMap<String, String>) {
    if config.keys().len() != 0 {
      self.min_position = config.get("min_position").expect("Restorable state needs 'min_position' var")
      .parse().expect("value 'min_position' can be parsed");
      self.gravity = config.get("gravity").expect("Restorable state needs 'gravity' var")
        .parse().expect("value 'gravity' can be parsed");
      self.max_position = config.get("max_position").expect("Restorable state needs 'max_position' var")
        .parse().expect("value 'max_position' can be parsed");
      self.max_speed = config.get("max_speed").expect("Restorable state needs 'max_speed' var")
        .parse().expect("value 'max_speed' can be parsed");
      self.goal_position = config.get("goal_position").expect("Restorable state needs 'goal_position' var")
        .parse().expect("value 'goal_position' can be parsed");
      self.goal_velocity = config.get("goal_velocity").expect("Restorable state needs 'goal_velocity' var")
        .parse().expect("value 'goal_velocity' can be parsed");
      self.force = config.get("force").expect("Restorable state needs 'force' var")
        .parse().expect("value 'force' can be parsed");
      self.max_steps = config.get("max_steps").expect("Restorable state needs 'max_steps' var")
        .parse().expect("value 'max_steps' can be parsed");
      if let Some(broader) = config.get("broader_start_position") {
        self.broader_start_position = broader.parse().expect("value 'broader_start_position' can be parsed");
      }
    }

  }
  fn get_name(&self) -> String {"MountainCar-v0".to_owned()}
  fn finalize(&mut self, _algo_name: &str, _eval_run_id: &str) {}
}

#[cfg(feature = "vis")]
use crate::vis::{VisualisableGymEnvironment, VertexInfo, GymVisRgbaS};
#[cfg(feature = "vis")]
use lyon_tessellation::{path::{Path, polygon::Polygon}, VertexBuffers, math::{Point, Box2D}, BuffersBuilder, FillOptions, StrokeOptions, StrokeTessellator, FillTessellator};
#[cfg(feature = "vis")]
use nalgebra::{Isometry2, Point2, Vector2};


#[cfg(feature = "vis")]
impl VisualisableGymEnvironment for MountainCar {
  fn get_area_size(&self) -> (u32, u32) { (600, 400) }
  fn get_fpms(&self) -> usize {(33.333333333) as usize}
  fn produce_observation(&mut self, zoom: f32) -> VertexBuffers<VertexInfo, u32> {
    let zoom64 = zoom as f64;
    let (screen_width_u, screen_height_u) = self.get_area_size();
    let screen_width = screen_width_u as f64 * zoom64;
    let screen_height = screen_height_u as f64 * zoom64;
    let _zoomed_size = 256.0 * zoom;
    let mut geometry: VertexBuffers<VertexInfo, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();

    let options = FillOptions::tolerance(0.1);
    let mut stroke_options = StrokeOptions::default();
    stroke_options.line_width = 1.2 * zoom;
    let mut base_line = StrokeOptions::default();
    base_line.line_width = 1.4 * zoom;
    // let builder_helper = vertex_info_builder(color_to_use);
    
    tessellator.tessellate_rectangle(&Box2D::new(Point::new(0.0, 0.0), Point::new(999999.0, 9999999.0)), &options,
      &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((255, 255, 255, 255)))).unwrap();

    let world_with = self.max_position - self.min_position;
    let scale = screen_width / world_with;

    let linspaced: Array1<f64> = ArrayBase::linspace(self.min_position, self.max_position, 100);
    let xs: Vec<f64> = linspaced.to_vec();
    let hill_line: Vec<Point> = xs.iter().map(|x| 
      Point::new(
        ((*x - self.min_position) * scale) as f32,
        (screen_height - (MountainCar::_height(*x) * scale)) as f32
      )
    ).collect();
    let mut hill_path_builder = Path::builder();
    for (i, p) in hill_line.iter().enumerate() {
      if i == 0 {
        hill_path_builder.begin(*p);
      } else {
        hill_path_builder.line_to(*p);
      }
    }
    // hill_path_builder.end(true);
    let hill_path = hill_path_builder.build();
    stroke_tessellator.tessellate_path(&hill_path, &base_line, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((0, 0, 0, 255)))).unwrap();

    let cart_y_pos = 0.0;
    let cart_x_pos = 0.0;
    let cart_width = 40.0 * zoom64;
    let cart_height = 20.0 * zoom64;

    // Draw Cart
    let l = -cart_width / 2.0;
    let r = cart_width / 2.0;
    let t = cart_height;
    let b = 0.0;
    let clearance = cart_width / 6.0;
    let _axle_offset = cart_height / 2.0;
    let cart_poly = vec![Point2::new(l, b), Point2::new(l, t), Point2::new(r, t), Point2::new(r, b),];
    let cart_translation = Isometry2::new(Vector2::new(cart_x_pos, cart_y_pos), (3.0 * self.position).cos());
    let translated_cart_poly: Vec<Point> = cart_poly.iter().map(|p| {
      let new_p = cart_translation * p;
      Point::new(
        (new_p.x + (self.position - self.min_position) * scale) as f32,
        (screen_height - (new_p.y + clearance + MountainCar::_height(self.position) * scale)) as f32
      )
    }).collect();
    let mut cart_path_builder = Path::builder();
    for (i, p) in translated_cart_poly.iter().enumerate() {
      if i == 0 {
        cart_path_builder.begin(*p);
      } else {
        cart_path_builder.line_to(*p);
      }
    }
    cart_path_builder.end(true);
    let cart_path = cart_path_builder.build();
    tessellator.tessellate_path(&cart_path, &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((0, 0, 255, 255)))).unwrap();

    let wheel_pos = vec![Point2::new(l + (cart_width /4.0), b), Point2::new(r - (cart_width /4.0), b)];
    let wheel_translation = Isometry2::new(Vector2::new(cart_x_pos, cart_y_pos), (3.0 * self.position).cos());
    let translated_wheel_pos: Vec<Point> = wheel_pos.iter().map(|p| {
      let new_p = wheel_translation * p;
      Point::new(
        (new_p.x + (self.position - self.min_position) * scale) as f32,
        (screen_height - (new_p.y + clearance + MountainCar::_height(self.position) * scale)) as f32
      )
    }).collect();
    let wheel_radius: f32 = cart_width as f32 / 6.0;
    // Left wheel
    tessellator.tessellate_circle(translated_wheel_pos[0], wheel_radius, &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((255, 0, 0, 255)))).unwrap();
    // Right wheel
    tessellator.tessellate_circle(translated_wheel_pos[1], wheel_radius, &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((255, 0, 0, 255)))).unwrap();


    let flagx = (self.goal_position - self.min_position) * scale;
    let flagy1 = MountainCar::_height(self.goal_position) * scale;
    let flagy2 = flagy1 + (50.0 * zoom64);
    
    tessellator.tessellate_rectangle(&Box2D::new(Point::new(flagx as f32, (screen_height - flagy2) as f32), Point::new(2.0 * zoom, 50.0 * zoom)), &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((0, 0, 255, 255)))).unwrap();

    let flag_poly = Polygon {closed: true, points: &[
      Point::new(flagx as f32, (screen_height - flagy2) as f32),
      Point::new(flagx as f32, (screen_height - (flagy2 - (10.0*zoom64))) as f32),
      Point::new((flagx + (25.0*zoom64)) as f32, (screen_height - (flagy2 - (5.0*zoom64))) as f32),
    ]};
    tessellator.tessellate_polygon(flag_poly, &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((204, 204, 0, 255)))).unwrap();

    let flag_poly2 = Polygon {closed: true, points: &[
      Point::new(flagx as f32, (screen_height - flagy2) as f32),
      Point::new(flagx as f32, (screen_height - (flagy2 - (10.0*zoom64))) as f32),
      Point::new((flagx + (25.0*zoom64)) as f32, (screen_height - (flagy2 - (5.0*zoom64))) as f32),
    ]};
    stroke_tessellator.tessellate_polygon(flag_poly2, &stroke_options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((0, 0, 0, 255)))).unwrap();

    

    geometry
  }
}


#[cfg(feature = "python")]
#[cfg(test)]
pub mod test {
  use crate::{GymEnvironment, env::python::PythonToRustGym};
  use pyo3::prelude::*;
  use rand_pcg::Pcg64Mcg;
  use rand::SeedableRng;
  use ndarray::ArrayBase;
  use super::MountainCar;

  // use rust_decimal::{prelude::ToPrimitive, Decimal};
  // fn cut_off(r1: f64) -> f64 {
  //   let rr1 = Decimal::from_f64_retain(r1).unwrap();
  //   let r1 = rr1.round_dp(8).to_f64().unwrap();
  //   return r1;
  // }
  // fn a_cut_off(o: Observation) -> Vec<f64> {
  //   let mut new: Vec<f64> = vec![];
  //   for v in o.iter() {
  //     new.push(cut_off(*v));
  //   }
  //   return new;
  // }

  // #[test]
  // fn python_to_rust_test() -> Result<(), PyErr> {
  //   let mut _rng = Pcg64Mcg::from_entropy();
  //   let seed_to_compare = 894;
  //   let mut py_cart = PythonToRustGym::from_str("MountainCar-v0", Some(seed_to_compare))?;
  //   py_cart.use_seed(seed_to_compare);
  //   let _py_initial = py_cart.reset();
  //   let mut rust_cart = MountainCar::new(Some(seed_to_compare));
  //   let mut rust_cart_diff = MountainCar::new(Some(seed_to_compare));
  //   rust_cart.use_seed(seed_to_compare);
  //   let _rust_initial = rust_cart.reset();


  //   // assert_eq!(py_initial, rust_initial);
  //   rust_cart_diff.use_seed(seed_to_compare);
  //   rust_cart_diff.reset();

  //   let action = ArrayBase::from(vec![1.0]).into_dyn();
  //   let mut done = false;
  //   let mut _step = 0;
  //   while !done {
  //     _step += 1;
  //     let _py_res = py_cart.step(&action);
  //     let rust_res = rust_cart.step(&action);
  //     let rust_res_diff = rust_cart_diff.step(&action);
  //     // assert_eq!(py_res, rust_res);
  //     assert_eq!(rust_res_diff, rust_res);
  //     done = rust_res.is_done;
  //   }

  //   Ok(())
  // }

  #[test]
  fn python_replay_equality() -> Result<(), PyErr> {

    // let mut rng = Pcg64Mcg::from_entropy();
    // let seed_to_compare = rng.next_u64();
    // let mut py_cart = PythonToRustGym::from_str("CartPole-v1", Some(seed_to_compare))?;
    // py_cart.use_seed(seed_to_compare);
    // py_cart.reset();
    // let mut rust_cart = MountainCar::new(Some(seed_to_compare), Some(RngType::Mt19937));
    // let mut rust_cart_diff = MountainCar::new(Some(seed_to_compare), Some(RngType::Pcg64Mcg));
    // let mut replay = load_run_convert_python("./GDH-a0dKzZXnZyDhWlgZb.tlr");
    // println!("GOT REPLAY {:?}", replay);
    // let episode = replay.episodes.index(50);
    // println!("GOT EPISODE 50 {}", episode.log.len());
    // for action in episode.log.iter() {
    //   let py_res = py_cart.step(&action);
    //   let rust_res = rust_cart.step(&action);
    //   let rust_res_diff = rust_cart_diff.step(&action);
    //   assert_eq!(py_res, rust_res);
    //   assert_ne!(rust_res_diff, rust_res);
    // }

    Ok(())
  }
}