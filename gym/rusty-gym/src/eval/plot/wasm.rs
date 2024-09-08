use plotters::{prelude::*, coord::Shift};
use plotters_canvas::CanvasBackend;

pub fn image_backend(target: &str, _size: (u32, u32)) -> DrawingArea<CanvasBackend, Shift> {
  let root = CanvasBackend::new(target).expect("Canvas exists").into_drawing_area();
  root.fill(&WHITE).unwrap();
  root
}
