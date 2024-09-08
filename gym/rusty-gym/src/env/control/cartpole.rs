use crate::{
  gym::{GymEnvironment, Step, Observation, Action},
  space::Space,
  util::rng::{UniRng, RngType, get_rng_for_type, uni_gen_range_f64}
};
use ndarray::ArrayBase;
use std::f64::consts::PI;


pub struct CartpoleEnv {
  rng: UniRng,
  rng_type: Option<RngType>,
  pub use_euler_kinematics: bool,
  action_space: Space,
  observation_space: Space,
  gravity: f64,
  mass_of_cart: f64,
  mass_of_pole: f64,
  combined_mass: f64,
  length: f64,
  polemass_length: f64,
  force_mag: f64,
  tau: f64,
  theta_threshold_radians: f64,
  x_threshold: f64,
  x: f64,
  x_dot: f64,
  theta: f64,
  theta_dot: f64,
  used_seed: u64,
  was_done: bool,
  step: usize,
  step_limit: usize
}

impl std::fmt::Debug for CartpoleEnv {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("Point")
       .field("mass_of_cart", &self.mass_of_cart)
       .finish()
  }
}

impl CartpoleEnv {
  pub fn new(seed: Option<u64>, rng_type: Option<RngType>) -> CartpoleEnv {
    let mass_of_cart = 1.0;
    let mass_of_pole = 0.1;
    let length = 0.5;
    let (rng, used_seed) = get_rng_for_type(&rng_type, seed);
    let mut env = CartpoleEnv {
      rng,
      rng_type: rng_type,
      used_seed,
      use_euler_kinematics: true,
      action_space: Space::Discrete(2),
      observation_space: Space::BoxedWithoutRange(vec![4]),
      gravity: 9.8,
      mass_of_cart,
      mass_of_pole,
      combined_mass: mass_of_cart + mass_of_pole,
      length,
      polemass_length: mass_of_pole * length,
      force_mag: 10.0,
      tau: 0.02,
      theta_threshold_radians: 12.0 * 2.0 * PI / 360.0,
      x_threshold: 2.4,
      x: 0.0,
      x_dot: 0.0,
      theta: 0.0,
      theta_dot: 0.0,
      step: 0,
      step_limit: 500,
      was_done: false
    };
    env.reset_state();
    env
  }
  pub fn reset_state(&mut self) {
    self.x = uni_gen_range_f64(&self.rng_type, &mut self.rng, -0.05, 0.05);
    self.x_dot = uni_gen_range_f64(&self.rng_type, &mut self.rng, -0.05, 0.05);
    self.theta = uni_gen_range_f64(&self.rng_type, &mut self.rng, -0.05, 0.05);
    self.theta_dot = uni_gen_range_f64(&self.rng_type, &mut self.rng, -0.05, 0.05);
    self.was_done = false;
    self.step = 0;
  }

  pub fn make_obs(&self) -> Observation {
    ArrayBase::from(vec![self.x, self.x_dot, self.theta, self.theta_dot]).into_dyn()
  }
}

impl Default for CartpoleEnv {
  fn default() -> CartpoleEnv {CartpoleEnv::new(None, None)}
}

impl GymEnvironment for CartpoleEnv {
  fn use_seed(&mut self, seed: u64) {
    let (new_rng, new_seed) = get_rng_for_type(&self.rng_type, Some(seed));
    self.rng = new_rng;
    self.used_seed = new_seed;
  }
  fn reset(&mut self) -> Observation {
    self.reset_state();
    self.make_obs()
  }

  fn step(&mut self, action: &Action) -> Step {
    let action_value = action[0].round();

    let force = if (action_value - 1.0f64).abs() < std::f64::EPSILON {self.force_mag} else {-self.force_mag};
    let cos_theta = self.theta.cos();
    let sin_theta = self.theta.sin();

    let temp = (force + self.polemass_length * self.theta_dot.powf(2.0) * sin_theta) / self.combined_mass;
    let theta_acceleration = (self.gravity * sin_theta - cos_theta * temp) /
      (self.length * (4.0 / 3.0 - self.mass_of_pole * cos_theta.powf(2.0) / self.combined_mass));
    let x_acceleration = temp - self.polemass_length * theta_acceleration * cos_theta / self.combined_mass;

    // TODO: removing this if can save a lot of cycles!
    if self.use_euler_kinematics {
      self.x += self.tau * self.x_dot;
      self.x_dot += self.tau * x_acceleration;
      self.theta += self.tau * self.theta_dot;
      self.theta_dot += self.tau * theta_acceleration;
    } else {
      self.x_dot += self.tau * x_acceleration;
      self.x += self.tau * self.x_dot;
      self.theta_dot += self.tau * theta_acceleration;
      self.theta += self.tau * self.theta_dot;
    }

    self.step += 1;
    let cart_is_outside_of_bounds = self.x < - self.x_threshold || self.x > self.x_threshold;
    let pole_is_tipped_over = self.theta < -self.theta_threshold_radians || self.theta > self.theta_threshold_radians;
    let is_done = self.step >= self.step_limit || cart_is_outside_of_bounds || pole_is_tipped_over;
    // println!("Episode is done because limit {}, outside bounds {} tipped over {}", self.step >= self.step_limit, cart_is_outside_of_bounds, pole_is_tipped_over);
    let reward = if !is_done {1.0} else if !self.was_done {self.was_done = true; 1.0} else {0.0};

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
impl ReplayableGymEnvironment for CartpoleEnv {
  fn get_used_seed(&mut self) -> u64 {self.used_seed}
  fn get_config(&mut self) -> HashMap<String, String> {
    let mut env_conf = HashMap::new();
    env_conf.insert("use_euler_kinematics".to_owned(), self.use_euler_kinematics.to_string());
    env_conf.insert("gravity".to_owned(), self.gravity.to_string());
    env_conf.insert("mass_of_cart".to_owned(), self.mass_of_cart.to_string());
    env_conf.insert("mass_of_pole".to_owned(), self.mass_of_pole.to_string());
    env_conf.insert("length".to_owned(), self.length.to_string());
    env_conf.insert("force_mag".to_owned(), self.force_mag.to_string());
    env_conf.insert("tau".to_owned(), self.tau.to_string());
    env_conf.insert("theta_threshold_radians".to_owned(), self.theta_threshold_radians.to_string());
    env_conf.insert("x_threshold".to_owned(), self.x_threshold.to_string());
    env_conf
  }
  fn load_config(&mut self, config: &HashMap<String, String>) {
    if config.keys().len() != 0 {

      self.use_euler_kinematics = config.get("use_euler_kinematics").expect("Restorable state needs 'use_euler_kinematics' var")
      .parse().expect("value 'use_euler_kinematics' can be parsed");
      self.gravity = config.get("gravity").expect("Restorable state needs 'gravity' var")
        .parse().expect("value 'gravity' can be parsed");
      self.mass_of_pole = config.get("mass_of_pole").expect("Restorable state needs 'mass_of_pole' var")
        .parse().expect("value 'mass_of_pole' can be parsed");
      self.mass_of_cart = config.get("mass_of_cart").expect("Restorable state needs 'mass_of_cart' var")
        .parse().expect("value 'mass_of_cart' can be parsed");
      self.combined_mass = self.mass_of_cart + self.mass_of_pole;
      self.length = config.get("length").expect("Restorable state needs 'length' var")
        .parse().expect("value 'length' can be parsed");
      self.polemass_length = self.mass_of_pole * self.length;
      self.force_mag = config.get("force_mag").expect("Restorable state needs 'force_mag' var")
        .parse().expect("value 'force_mag' can be parsed");
      self.tau = config.get("tau").expect("Restorable state needs 'tau' var")
        .parse().expect("value 'tau' can be parsed");
      self.theta_threshold_radians = config.get("theta_threshold_radians").expect("Restorable state needs 'theta_threshold_radians' var")
        .parse().expect("value 'theta_threshold_radians' can be parsed");
      self.x_threshold = config.get("x_threshold").expect("Restorable state needs 'x_threshold' var")
        .parse().expect("value 'x_threshold' can be parsed");
    }

  }
  fn get_name(&self) -> String {"CartPole-v0".to_owned()}
  fn finalize(&mut self, _algo_name: &str, _eval_run_id: &str) {}
}

#[cfg(feature = "reset")]
use crate::reset::ResettableGymEnvironment;
#[cfg(feature = "reset")]
impl ResettableGymEnvironment for CartpoleEnv {
  fn restore_state(&mut self, state: &HashMap<String, String>) {
    self.x = state.get("x").expect("Restorable state needs 'x' var")
      .parse().expect("value 'x' can be parsed");
    self.x_dot = state.get("x_dot").expect("Restorable state needs 'x_dot' var")
      .parse().expect("value 'x_dot' can be parsed");
    self.theta = state.get("theta").expect("Restorable state needs 'theta' var")
      .parse().expect("value 'x' can be parsed");
    self.theta_dot = state.get("theta_dot").expect("Restorable state needs 'theta_dot' var")
      .parse().expect("value 'theta_dot' can be parsed");
    self.step = state.get("step").expect("Restorable state needs 'step' var")
      .parse().expect("value 'step' can be parsed");
    self.was_done = state.get("was_done").expect("Restorable state needs 'was_done' var")
      .parse().expect("value 'was_done' can be parsed");
  }
  fn get_restorable_state(&mut self) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("x".to_owned(), self.x.to_string());
    map.insert("x_dot".to_owned(), self.x.to_string());
    map.insert("theta".to_owned(), self.x.to_string());
    map.insert("theta_dot".to_owned(), self.x.to_string());
    map.insert("step".to_owned(), self.step.to_string());
    map.insert("was_done".to_owned(), self.was_done.to_string());
    map
  }
}

#[cfg(feature = "vis")]
use crate::vis::{VisualisableGymEnvironment, VertexInfo, GymVisRgbaS};
#[cfg(feature = "vis")]
use lyon_tessellation::{path::Path, VertexBuffers, math::{Box2D, Point}, BuffersBuilder, FillOptions, FillTessellator};
#[cfg(feature = "vis")]
use nalgebra::{Isometry2, Translation2, Rotation2, Point2, Vector2};


#[cfg(feature = "vis")]
impl VisualisableGymEnvironment for CartpoleEnv {
  fn get_area_size(&self) -> (u32, u32) { (100, 67) }
  fn get_fpms(&self) -> usize {(1000.0 * self.tau) as usize}
  fn produce_observation(&mut self, zoom: f32) -> VertexBuffers<VertexInfo, u32> {
    let (screen_width_u, screen_height_u) = self.get_area_size();
    let screen_width = screen_width_u as f32;
    let screen_height = screen_height_u as f32;
    let _zoomed_size = 256.0 * zoom;
    let mut geometry: VertexBuffers<VertexInfo, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let options = FillOptions::tolerance(0.1);
    // let builder_helper = vertex_info_builder(color_to_use);
    
    tessellator.tessellate_rectangle(&Box2D::new(Point::new(0.0, 0.0), Point::new(screen_width * zoom, screen_height * zoom)), &options,
      &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((0, 255, 0, 255)))).unwrap();

    let _world_with = self.x_threshold * 2.0;
    let scale = 600.0 / 400.0;
    let cart_y_pos = 100.0;
    let cart_x_pos = self.x * scale + 600.0 /2.0;
    let pole_width = 10.0;
    let pole_length = scale * (2.0 * self.length);
    let cart_width = 50.0;
    let cart_height = 30.0;

    // Draw Cart
    let l = -cart_width / 2.0;
    let r = cart_width / 2.0;
    let t = cart_height / 2.0;
    let b = -cart_height / 2.0;
    let _axle_offset = cart_height / 2.0;
    let cart_poly = vec![Point2::new(l, b), Point2::new(l, t), Point2::new(r, t), Point2::new(r, b),];
    let cart_translation = Isometry2::new(Vector2::new(cart_x_pos, cart_y_pos), 0.0);
    let translated_cart_poly: Vec<Point> = cart_poly.iter().map(|p| {
      let new_p = cart_translation * p;
      Point::new(new_p.x as f32, new_p.y as f32)
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


    // Draw Pole
    let left = -pole_width / 2.0;
    let right = pole_width / 2.0;
    let top = pole_length - pole_width / 2.0;
    let bottom = -pole_width / 2.0;
    let axle_offset = cart_height / 2.0;
    let pole_poly = vec![Point2::new(left, bottom), Point2::new(left, top), Point2::new(right, top), Point2::new(right, bottom),];

    // let pole_translation = Isometry2::new(Vector2::new(cart_x_pos, cart_y_pos - axle_offset), 90.0-self.theta);
    let pole_translation = Translation2::new(cart_x_pos, cart_y_pos - axle_offset);
    let pole_rotation = Rotation2::new(90.0-self.theta);
    let translated_pole_poly: Vec<Point> = pole_poly.iter().map(|p| {
      let new_p = pole_translation * pole_rotation * p;
      Point::new(new_p.x as f32, new_p.y as f32)
    }).collect();
    let mut pole_path_builder = Path::builder();
    for (i, p) in translated_pole_poly.iter().enumerate() {
      if i == 0 {
        pole_path_builder.begin(*p);
      } else {
        pole_path_builder.line_to(*p);
      }
    }
    pole_path_builder.end(true);
    let pole_path = pole_path_builder.build();
    tessellator.tessellate_path(&pole_path, &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((200, 150, 80, 255)))).unwrap();

    let wheel_pos_y: f32 = cart_y_pos as f32 + axle_offset as f32;
    let wheel_radius: f32 = cart_width as f32 / 6.0;
    // Left wheel
    tessellator.tessellate_circle(Point::new(cart_x_pos as f32 - cart_width as f32 / 4.0, wheel_pos_y), wheel_radius, &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((255, 0, 0, 255)))).unwrap();
    // Right wheel
    tessellator.tessellate_circle(Point::new(cart_x_pos as f32 + cart_width as f32 / 4.0, wheel_pos_y), wheel_radius, &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((255, 0, 0, 255)))).unwrap();

    // Track
    let mut track_path_builder = Path::builder();
    let _track_y = cart_y_pos - axle_offset;
    track_path_builder.begin(Point::new(0.0, cart_y_pos as f32));
    track_path_builder.line_to( Point::new(screen_width, cart_y_pos as f32));
    track_path_builder.line_to( Point::new(screen_width, cart_y_pos as f32 + 2.0));
    track_path_builder.line_to( Point::new(0.0, cart_y_pos as f32 + 2.0));
    track_path_builder.end(true);
    let track_path = track_path_builder.build();
    tessellator.tessellate_path(&track_path, &options, &mut BuffersBuilder::new(&mut geometry, GymVisRgbaS((0, 0, 0, 255)))).unwrap();

    geometry
  }
}


#[cfg(feature = "python")]
#[cfg(test)]
pub mod test {
  use crate::{GymEnvironment, env::python::PythonToRustGym, Observation, util::rng::RngType};
  use pyo3::prelude::*;
  use rand_pcg::Pcg64Mcg;
  use rand::SeedableRng;
  use ndarray::ArrayBase;
  use super::CartpoleEnv;

  use rust_decimal::{prelude::ToPrimitive, Decimal};
  fn cut_off(r1: f64) -> f64 {
    let rr1 = Decimal::from_f64_retain(r1).unwrap();
    let r1 = rr1.round_dp(8).to_f64().unwrap();
    return r1;
  }
  fn a_cut_off(o: Observation) -> Vec<f64> {
    let mut new: Vec<f64> = vec![];
    for v in o.iter() {
      new.push(cut_off(*v));
    }
    return new;
  }

  // #[test]
  // fn python_to_rust_test() -> Result<(), PyErr> {
  //   let mut _rng = Pcg64Mcg::from_entropy();
  //   let seed_to_compare = 894;
  //   let mut py_cart = PythonToRustGym::from_str("CartPole-v1", Some(seed_to_compare))?;
  //   py_cart.use_seed(seed_to_compare);
  //   let py_initial = py_cart.reset();
  //   let mut rust_cart = CartpoleEnv::new(Some(seed_to_compare), Some(RngType::Mt19937));
  //   let mut rust_cart_diff = CartpoleEnv::new(Some(seed_to_compare), Some(RngType::Pcg64Mcg));
  //   rust_cart.use_seed(seed_to_compare);
  //   let rust_initial = rust_cart.reset();
  //   println!("GOT {:?} vs {:?}", a_cut_off(py_initial), a_cut_off(rust_initial));


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
  //     assert_ne!(rust_res_diff, rust_res);
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
    // let mut rust_cart = CartpoleEnv::new(Some(seed_to_compare), Some(RngType::Mt19937));
    // let mut rust_cart_diff = CartpoleEnv::new(Some(seed_to_compare), Some(RngType::Pcg64Mcg));
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