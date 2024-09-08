use plotters::{prelude::*, coord::Shift};

pub fn image_backend(target: &str, size: (u32, u32)) -> DrawingArea<BitMapBackend, Shift> {
  let root = BitMapBackend::new(target, size).into_drawing_area();
  root.fill(&WHITE).unwrap();
  root
}
