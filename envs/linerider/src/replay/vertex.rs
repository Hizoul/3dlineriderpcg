use egui::{Ui, emath::{Pos2, vec2}, color::Color32};
use epaint::{Mesh, TextureId, Shape};
use rusty_gym::vis::VertexInfo;
use lyon::tessellation::{VertexBuffers};

pub fn render_vertex_to_egui(ui: &mut Ui, raster_size: (u32, u32), geometry: &VertexBuffers<VertexInfo, u32>) {
  let allocated_rect_outer = ui.allocate_space(vec2(raster_size.0 as f32, raster_size.1 as f32));
  let allocated_rect = allocated_rect_outer.1;
  let mut tringles = Mesh::with_texture(TextureId::Managed(0));
  for vertex in geometry.vertices.iter() {
    let new_pos = Pos2{x: allocated_rect.min.x + vertex.0.0, y: allocated_rect.min.y+vertex.0.1};
    tringles.colored_vertex(new_pos, Color32::from_rgba_premultiplied(vertex.1.0, vertex.1.1, vertex.1.2, vertex.1.3));
  }
  for i in 0..(geometry.indices.len()/3) {
    let start_from = i * 3;
    tringles.add_triangle(geometry.indices[start_from], geometry.indices[start_from+1], geometry.indices[start_from+2]);
  }

  ui.painter().add(Shape::Mesh(tringles));
}
