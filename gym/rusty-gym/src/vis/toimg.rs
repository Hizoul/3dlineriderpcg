use std::collections::HashMap;
use lyon_tessellation::{VertexBuffers};
use crate::{vis::{VertexInfo, VisualisableGymEnvironment, GymVisPos2}, EpisodeData};

fn point_in_edge(point_x: f32, point_y: f32, v0: &GymVisPos2, v1: &GymVisPos2) -> bool {
  ((point_x - v0.0) * (v1.1 - v0.1) - (point_y - v0.1) * (v1.0 - v0.0)) >= 0.0
}

fn point_in_triangle(point_x: f32, point_y: f32, v0: &GymVisPos2, v1: &GymVisPos2, v2: &GymVisPos2) -> bool {
  let mut is_inside = true;
  is_inside &= point_in_edge(point_x, point_y, v0, v1);
  is_inside &= point_in_edge(point_x, point_y, v1, v2);
  is_inside &= point_in_edge(point_x, point_y, v2, v0);
  is_inside
}

pub type RasterizedImage = HashMap<(u32, u32), [u8; 4]>;

pub fn rasterize_naive(raster_width: u32, raster_height: u32, geometry: &VertexBuffers<VertexInfo, u32>) -> RasterizedImage {
  let mut rasterized: RasterizedImage = HashMap::new();
  for y in 0..raster_height {
    for x in 0..raster_width {
      let x_f = x as f32;
      let y_f = y as f32;
      let pixel_pos = (x, y);
      for i in 0..(geometry.indices.len()/3) {
        let start_from = i * 3;
        let triangle = (geometry.indices[start_from], geometry.indices[start_from+1], geometry.indices[start_from+2]);
        let v0 = &geometry.vertices[triangle.0 as usize];
        let v1 = &geometry.vertices[triangle.1 as usize];
        let v2 = &geometry.vertices[triangle.2 as usize];
        if point_in_triangle(x_f, y_f, &v0.0, &v1.0, &v2.0) {
          let color = [v0.1.0, v0.1.1, v0.1.2, v0.1.3];
          rasterized.insert(pixel_pos, color);
        }
      }
    }
  }
  rasterized
}

pub fn rasterize_naive_vec(raster_width: u32, raster_height: u32, geometry: &[VertexBuffers<VertexInfo, u32>]) -> Vec<RasterizedImage> {
  let mut rasterized  = Vec::new();
  for g in geometry {
    rasterized.push(rasterize_naive(raster_width, raster_height, g));
  }
  rasterized
}

use image::{DynamicImage, GenericImage, Rgba, Frame, codecs::gif::{GifEncoder}};

pub fn rasterized_to_dynamic_image(raster_width: u32, raster_height: u32, image: &RasterizedImage) -> DynamicImage {
  let mut dyim = DynamicImage::new_rgba8(raster_width as u32, raster_height as u32);
  for ((x, y), color) in image {
    dyim.put_pixel(*x, *y, Rgba(*color));
  }
  dyim
}

pub fn save_rasterized_to_file(path: &str, raster_width: u32, raster_height: u32, image: &RasterizedImage) {
  let dyim = rasterized_to_dynamic_image(raster_width, raster_height, image);
  dyim.save(path).unwrap();
}

pub fn save_rasterized_gif(path: &str, raster_width: u32, raster_height: u32, images: &[RasterizedImage]) {
  let mut frames: Vec<Frame> = Vec::with_capacity(images.len());
  for image in images {
    let dynamic_image = rasterized_to_dynamic_image(raster_width, raster_height, image);
    frames.push(Frame::new(dynamic_image.as_rgba8().unwrap().clone()));
  }
  let gif_file = std::fs::File::create(path).unwrap();
  let mut encoder = GifEncoder::new(gif_file);
  encoder.encode_frames(frames.into_iter()).unwrap();
}

pub fn episode_to_geometry(env: &mut impl VisualisableGymEnvironment, episode: &EpisodeData, zoom_opt: Option<f32>) -> Vec<VertexBuffers<VertexInfo, u32>> {
  let zoom = zoom_opt.unwrap_or(1.0);
  let mut obs = Vec::new();
  env.use_seed(episode.seed);
  obs.push(env.produce_observation(zoom));
  for action in episode.log.iter() {
    env.step(action);
    obs.push(env.produce_observation(zoom));
  }
  obs
}

pub fn episode_to_gif(path: &str, env: &mut impl VisualisableGymEnvironment, episode: &EpisodeData, zoom_opt: Option<f32>) {
  let episode_as_geometry = episode_to_geometry(env, episode, zoom_opt);
  let zoom = zoom_opt.unwrap_or(1.0);
  let env_size = env.get_area_size();
  let raster_width = (env_size.0 as f32 * zoom) as u32;
  let raster_height = (env_size.1 as f32 * zoom) as u32;
  let rasterized = rasterize_naive_vec(raster_width, raster_height, &episode_as_geometry);
  save_rasterized_gif(path, raster_width, raster_height, &rasterized);
}