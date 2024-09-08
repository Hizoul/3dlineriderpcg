pub mod registry;
use lyon_tessellation::{VertexBuffers, math::Point, FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor};
use crate::{ReplayableGymEnvironment, EpisodeData, Reward};

// x, y coordinates + color
pub type GymVisPos2 = (f32, f32);
pub type GymVisRgba = (u8, u8, u8, u8);
pub type VertexInfo = (GymVisPos2, GymVisRgba);
pub struct GymVisRgbaS(pub GymVisRgba);

impl FillVertexConstructor<VertexInfo> for GymVisRgbaS {
  fn new_vertex(&mut self, vertex: FillVertex) -> VertexInfo {
    let pos = vertex.position();
    (
      (pos.x, pos.y), self.0
    )
  }
}

impl StrokeVertexConstructor<VertexInfo> for GymVisRgbaS {
  fn new_vertex(&mut self, vertex: StrokeVertex) -> VertexInfo {
    let pos = vertex.position();
    (
      (pos.x, pos.y), self.0
    )
  }
}

pub trait VisualisableGymEnvironment: ReplayableGymEnvironment + std::fmt::Debug {
  fn get_area_size(&self) -> (u32, u32);
  fn get_fpms(&self) -> usize;
  fn produce_observation(&mut self, zoom: f32) -> VertexBuffers<VertexInfo, u32>;
}

pub fn vertex_info_builder(color_to_use: GymVisRgba) -> impl Fn(Point) -> VertexInfo {move |pos: Point| ((pos.x, pos.y), color_to_use)}

pub fn episode_to_reward_vec_v(env: &mut Box<dyn VisualisableGymEnvironment>, episode: &EpisodeData) -> Vec<Reward> {
  let mut rewards = Vec::with_capacity(episode.log.len());
  if let Some(env_params) = &episode.env_params {
    env.load_config(env_params);
  }
  env.use_seed(episode.seed);
  env.reset(); // TODO: determine if necessary
  for action in episode.log.iter() {
    let res = env.step(action);
    rewards.push(res.reward);
  }
  rewards
}
pub fn episode_to_reward_vec_v_rep(env: &mut Box<dyn ReplayableGymEnvironment>, episode: &EpisodeData) -> Vec<Reward> {
  let mut rewards = Vec::with_capacity(episode.log.len());
  if let Some(env_params) = &episode.env_params {
    env.load_config(env_params);
  }
  env.use_seed(episode.seed);
  env.reset(); // TODO: determine if necessary
  for action in episode.log.iter() {
    let res = env.step(action);
    rewards.push(res.reward);
  }
  rewards
}

#[cfg(feature = "vis-toimg")]
pub mod toimg;
#[cfg(feature = "vis-toimg")]
pub use toimg::*;