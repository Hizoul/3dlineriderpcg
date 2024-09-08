use plotters_backend::{
    BackendColor, BackendStyle, BackendCoord, DrawingBackend, DrawingErrorKind,
};
use epaint::{
  CircleShape, PathShape, text::{LayoutJob, TextFormat}, TextShape, RectShape //, RectShape, text::{LayoutJob, TextFormat}, TextShape
};
use bevy_egui::egui::{Ui, Shape, Stroke, Color32, emath::{Rect, Pos2, pos2, vec2}};
use std::sync::{Arc, Mutex};

pub type EguiPlottersPixel = Shape;
pub type EguiPlottersPixels = Vec<Shape>;
pub type EguiPlottersPixelsLocked = Arc<Mutex<EguiPlottersPixels>>;

pub type EguiPlottersText = (Pos2, String, Color32);
pub type EguiPlottersTexts = Vec<EguiPlottersText>;
pub type EguiPlottersTextsLocked = Arc<Mutex<EguiPlottersTexts>>;

pub struct EguiBackend {
  size: (u32, u32),
  cmds_to_paint: EguiPlottersPixelsLocked,
  _text_to_paint: EguiPlottersTextsLocked
}

impl EguiBackend {
  pub fn new(size: (u32, u32), cmds_to_paint: EguiPlottersPixelsLocked, _text_to_paint: EguiPlottersTextsLocked) -> EguiBackend {
    EguiBackend {size, cmds_to_paint, _text_to_paint}
  }
}


pub fn backendcolor_to_srgba<S: BackendStyle>(style: S) -> Color32 {
  let color = style.color();
  Color32::from_rgba_premultiplied(color.rgb.0, color.rgb.1, color.rgb.2, (color.alpha * 255.0) as u8)
}

pub fn backendcord_to_pos2(coord: BackendCoord) -> Pos2 {
  pos2(coord.0 as f32, coord.1 as f32)
}

impl DrawingBackend for EguiBackend {
  type ErrorType = std::io::Error;

  fn get_size(&self) -> (u32, u32) {
    self.size
  }

  fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
    Ok(())
  }

  fn present(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
    Ok(())
  }

  fn draw_pixel(&mut self, point: BackendCoord, style: BackendColor) -> Result<(), DrawingErrorKind<std::io::Error>> {
    let color = style;
    if color.alpha == 0.0 || color.rgb == (255, 255, 255) {
      return Ok(());
    }
    let mut cmds = self.cmds_to_paint.lock().unwrap();
    let cmd = Shape::rect_filled(
      Rect::from_min_max(
        backendcord_to_pos2(point), 
        backendcord_to_pos2((point.0+1, point.1+1))),
      0.0,
      backendcolor_to_srgba(style)
    );
    cmds.push(cmd);
    Ok(())
  }
  fn draw_line<S: BackendStyle>(&mut self, from: BackendCoord, to: BackendCoord, style: &S) -> Result<(), DrawingErrorKind<std::io::Error>> {
    let color = style.color();
    let mut cmds = self.cmds_to_paint.lock().unwrap();
    cmds.push(
      Shape::LineSegment {
        points: [backendcord_to_pos2(from), backendcord_to_pos2(to)],
        stroke: Stroke {width: style.stroke_width() as f32, color: backendcolor_to_srgba(color)}
      }
    );
    Ok(())
  }

  fn draw_rect<S: BackendStyle>(&mut self, upper_left: BackendCoord, bottom_right: BackendCoord, style: &S, fill: bool) -> Result<(), DrawingErrorKind<std::io::Error>> {
    let mut cmds = self.cmds_to_paint.lock().unwrap();
    let color = backendcolor_to_srgba(style.color());
    let stroke_width = style.stroke_width() as f32;
    let rect = Rect::from_min_max(backendcord_to_pos2(upper_left), backendcord_to_pos2(bottom_right));
    cmds.push(if fill {
      Shape::rect_filled(rect, 0.0, color)
    } else {
      Shape::rect_stroke(rect, 0.0, Stroke::new(stroke_width, color))
    });
    Ok(())
  }

  fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(&mut self, path: I, style: &S) -> Result<(), DrawingErrorKind<std::io::Error>> {
    let points: Vec<Pos2> = path.into_iter().map(backendcord_to_pos2).collect();
    let mut cmds = self.cmds_to_paint.lock().unwrap();
    let color = backendcolor_to_srgba(style.color());
    let stroke_width = style.stroke_width() as f32;
    cmds.push(Shape::Path(PathShape {points, closed: false, fill: Color32::TRANSPARENT, stroke: Stroke::new(stroke_width, color)}));
    Ok(())
  }

  fn draw_circle<S: BackendStyle>(&mut self, center: BackendCoord, radius: u32, style: &S, fill: bool) -> Result<(), DrawingErrorKind<std::io::Error>> {
    let mut cmds = self.cmds_to_paint.lock().unwrap();
    let color = backendcolor_to_srgba(style.color());
    let stroke_width = style.stroke_width() as f32;
    let center_pos = backendcord_to_pos2(center);
    cmds.push(if fill {
      Shape::circle_filled(center_pos, radius as f32, color)
    } else {
      Shape::circle_stroke(center_pos, radius as f32, Stroke::new(stroke_width, color))
    });
    Ok(())
  }

  #[cfg(target_arch = "wasm32")]
  fn draw_text<TStyle: plotters_backend::BackendTextStyle>(&mut self, text: &str, style: &TStyle, pos: BackendCoord) -> Result<(), DrawingErrorKind<std::io::Error>> {
    let mut texts = self._text_to_paint.lock().unwrap();
    let color = backendcolor_to_srgba(style.color());
    
    texts.push((backendcord_to_pos2(pos), text.to_owned(), color));
    
    Ok(())
  }

}

pub fn translate_and_paint_cmds_to_area(ui: &mut Ui, size: (u32, u32), cmds: &[EguiPlottersPixel], texts: &[EguiPlottersText]) -> Rect {
  let allocated_rect_outer = ui.allocate_space(vec2(size.0 as f32, size.1 as f32));
  let allocated_rect = allocated_rect_outer.1;
  let bg_fill = Shape::rect_filled(allocated_rect,
    0.0,
    Color32::from_rgba_premultiplied(255, 255, 255, 255)
  );
  ui.painter().add(bg_fill);
  let add_x = allocated_rect.min.x;
  let add_y = allocated_rect.min.y;
  let add_v = vec2(add_x, add_y);
  for cmd in cmds {
    let new_cmd = match cmd {
      Shape::Circle(CircleShape { center, radius, fill, stroke }) => {
        Shape::Circle(CircleShape  { center: *center + add_v, radius: *radius, fill: *fill, stroke: *stroke})
      },
      Shape::LineSegment {points, stroke} => {
        Shape::LineSegment{points: [points[0]+add_v, points[1]+add_v], stroke: *stroke}
      },
      Shape::Path(PathShape {points, closed, fill, stroke}) => {
        let translated_points: Vec<Pos2> = points.iter().map(|p| *p + add_v).collect();
        Shape::Path(PathShape {points: translated_points, closed: *closed, fill: *fill, stroke: *stroke})
      },
      Shape::Rect(RectShape { rect,rounding,fill,stroke, fill_texture_id, uv }) => {
        let translated_rect = Rect::from_min_max(rect.min + add_v, rect.max + add_v);
        Shape::Rect(RectShape {rect: translated_rect, rounding: *rounding, fill: *fill, stroke: *stroke, fill_texture_id: *fill_texture_id, uv: *uv})
      },
      _ => {
        cmd.clone()
      }
    };
    ui.painter().add(new_cmd);
  }
   ui.fonts(|fonts| {
    for text in texts {
      let mut job = LayoutJob::default();
      job.append(&text.1, 0.0, TextFormat::default());
      let galleys = fonts.layout_job(job);
      let textshape = TextShape::new(text.0+add_v, galleys);
      ui.painter().add(Shape::Text(textshape));
    }
  });
  allocated_rect
}