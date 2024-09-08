use crate::space::Space;
use crate::gym::{GymEnvironment, Step, Observation, Action};
use ndarray::ArrayBase;

#[derive(Debug, Clone)]
pub struct EnvZeroOrOne {
  action_space: Space,
  observation_space: Space,
  tick: i64,
  render_size: u32
}

impl EnvZeroOrOne {
  pub fn new() -> EnvZeroOrOne {
    EnvZeroOrOne {
      action_space: Space::Discrete(2),
      observation_space: Space::BoxedWithoutRange(vec![3]),
      tick: 0,
      render_size: 32
    }
  }

  pub fn is_odd(&self) -> bool {
    self.tick % 2 == 0
  }

  pub fn make_obs(&self) -> Observation {
    ArrayBase::from(vec![0.0f64, if self.is_odd() {1.0} else {0.0}, 0.0]).into_dyn()
  }
}

impl Default for EnvZeroOrOne {
  fn default() -> EnvZeroOrOne {EnvZeroOrOne::new()}
}

impl GymEnvironment for EnvZeroOrOne {
  fn use_seed(&mut self, _seed: u64) {self.reset();}
  fn reset(&mut self) -> Observation {
    self.tick = 0;
    self.make_obs()
  }

  fn step(&mut self, action: &Action) -> Step {
    let action_value = action[0].round();
    let expectation = if self.is_odd() {1.0} else {0.0};
    self.tick += 1;
    let obs = self.make_obs();
    Step {
      obs,
      reward: if (action_value - expectation).abs() < f64::EPSILON {1.0} else {-1.0},
      is_done: self.tick > 10,
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
impl ReplayableGymEnvironment for EnvZeroOrOne {
  fn get_used_seed(&mut self) -> u64 {0}
  fn get_config(&mut self) -> HashMap<String, String> {HashMap::new()}
  fn load_config(&mut self, _config: &HashMap<String, String>) {}
  fn get_name(&self) -> String {"zero_or_one".to_owned()}
  fn finalize(&mut self, _algo_name: &str, _eval_run_id: &str) {}
}



#[cfg(feature = "vis")]
use crate::vis::{VisualisableGymEnvironment, VertexInfo, GymVisRgbaS};
#[cfg(feature = "vis")]
use lyon_tessellation::{VertexBuffers, BuffersBuilder, FillOptions, FillTessellator, math::{Box2D, Point}};


#[cfg(feature = "vis")]
impl VisualisableGymEnvironment for EnvZeroOrOne {
  fn get_area_size(&self) -> (u32, u32) { (self.render_size, self.render_size) }
  fn get_fpms(&self) -> usize {1000}
  fn produce_observation(&mut self, zoom: f32) -> VertexBuffers<VertexInfo, u32> {
    let zoomed_size = self.render_size as f32 * zoom;
    let mut geometry: VertexBuffers<VertexInfo, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let options = FillOptions::tolerance(0.1);
    let color_to_use = if self.is_odd() {(0, 255, 0, 255)} else {(255, 0, 0, 255)};
    let color = GymVisRgbaS(color_to_use);
    // let builder_helper = vertex_info_builder(color_to_use);
    
    tessellator.tessellate_rectangle(&Box2D::new(Point::new(0.0, 0.0,), Point::new(zoomed_size, zoomed_size)), &options,
    &mut BuffersBuilder::new(&mut geometry, color)).unwrap();

    geometry
  }
}


#[cfg(test)]
pub mod test {
  use super::EnvZeroOrOne;
  use crate::{GymEnvironment, VisualisableGymEnvironment};
  use plotters::prelude::*;
  use ndarray::ArrayBase;
  #[test]
  fn simpledrawtest() {
    let mut env = EnvZeroOrOne::default();
    let area = BitMapBackend::gif("../test.gif", env.get_area_size(), 30).unwrap().into_drawing_area();
    let mut is_done = false;
    while !is_done {
      area.fill(&BLACK).unwrap();
      // env.draw_to_area(&mut area);
      area.present().unwrap();
      let step = env.step(&ArrayBase::from(vec![0.0]).into_dyn());
      is_done = step.is_done;
    }
  }
}

