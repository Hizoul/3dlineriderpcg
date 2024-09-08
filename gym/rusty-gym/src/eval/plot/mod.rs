#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;



use plotters::{prelude::*, coord::{Shift, types::{RangedCoordi64, RangedCoordf64},ranged1d::{ValueFormatter}}, style::{RGBAColor}};


use crate::{RewardVector, Reward};

pub fn mean(arr: &[f64]) -> i32 {
  if arr.is_empty() {
    return 0;
  }
  let mut sum = 0.0;
  for entry in arr {
    sum += entry;
  }
  (sum / arr.len() as f64) as i32
}

pub fn get_color(i: usize) -> RGBAColor {
  let colors: Vec<Box<dyn Color>> = vec![
    Box::new(RED), Box::new(GREEN), Box::new(BLUE), Box::new(CYAN), Box::new(BLACK), Box::new(YELLOW)
  ];
  colors[i % colors.len()].to_rgba()
}

#[macro_export]
macro_rules! to_line_series {
    ($data:expr) => {
      {
        let mut lines = Vec::with_capacity($data.len());
        let mut i = 0;
        for entry in $data {
          lines.push((i, entry));
          i += 1;
        }
        lines
      }
    };
}

#[macro_export]
macro_rules! to_line_series_float {
    ($data:expr) => {
      {
        let mut lines = Vec::with_capacity($data.len());
        let mut i = 0.0;
        for entry in $data {
          lines.push((i, entry));
          i += 1.0;
        }
        lines
      }
    };
}


pub fn make_chart<'a, 'b, XT, YT, DB: DrawingBackend, X: Ranged<ValueType = XT> + ValueFormatter<XT>, Y: Ranged<ValueType = YT> + ValueFormatter<YT>>(draw_backend: &'a DrawingArea<DB, Shift>, zoom: f64, name: &str, x_range: X, y_range: Y, x_desc: &str, y_desc: &str) -> ChartContext<'a, DB, Cartesian2d<X, Y>> {
  let mut chart = ChartBuilder::on(draw_backend)
  .x_label_area_size(43 * zoom as u32)
  .y_label_area_size(53 * zoom as u32)
  .margin(12 * zoom as u32)
  .caption(name, ("Arial", (30.0 * zoom) as f32).into_font()).build_cartesian_2d(x_range, y_range).unwrap();
  let color = BLACK.to_rgba();
  let color2 = BLACK.to_rgba();
  let mut mesh_configurer = chart.configure_mesh();
  mesh_configurer.y_desc(y_desc)
    .x_desc(x_desc)
    .x_label_style(("Arial", 12.0 * zoom).into_font())
    .y_label_style(("Arial", 12.0 * zoom).into_font())
    .axis_desc_style(("Arial", 20.0 * zoom).into_font());
  if zoom > 1.0 {
    mesh_configurer.bold_line_style(ShapeStyle{color,stroke_width: 1 * ((zoom * 0.3) as u32),filled: true})
    .light_line_style(ShapeStyle{color: color2,stroke_width: 1 * ((zoom * 0.3) as u32),filled: true});
  }
  mesh_configurer.draw().unwrap();
  chart
}

// pub fn dra_lines<'a, 'b, XT: Clone, YT: Clone, DB: DrawingBackend, X: Ranged<ValueType = XT> + ValueFormatter<XT>, Y: Ranged<ValueType = YT> + ValueFormatter<YT>>(chart: &mut ChartContext<'a, DB, Cartesian2d<X, Y>>, line_data: Vec<YT>) {

//     let color = get_color(0);
//     chart.draw_series(LineSeries::new(
//       line_data,
//       ShapeStyle{color,stroke_width: 1,filled: true})).unwrap()
//       .label("B")
//       .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &get_color(0)));
// }



#[macro_export]
macro_rules! draw_lines {
    ($chart:expr, $line_data:expr) => {
      let mut i = 0;
      for line in $line_data {
        let color = get_color(i);
        let sub_i = i;
        $chart.draw_series(LineSeries::new(
          line.1.clone(),
          ShapeStyle{color,stroke_width: 1,filled: true})).unwrap()
          .label(line.0.as_str())
          .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &get_color(sub_i)));
        i += 1;
      }
    };
}

#[macro_export]
macro_rules! draw_candles {
    ($chart:expr, $candles:expr) => {
      let mut i = 0;
      for line in $candles {
        let color = get_color(i);
        let sub_i = i;
        $chart.draw_series(
          line.1.iter().map(|x| CandleStick::new(x.0, x.1, x.2, x.3, x.4, &color, &color, 9))
        ).unwrap()
          .label(line.0.as_str())
          .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &get_color(sub_i)));
        i += 1;
      }
    };
}
#[macro_export]
macro_rules! draw_candle {
    ($chart:expr, $candles:expr) => {
      let mut i = 0;
      for x in $candles {
        let color = get_color(i);
        let sub_i = i;
        $chart.draw(
          &CandleStick::new(x.0, x.1, x.2, x.3, x.4, &color, &color, 9)
        ).unwrap();
        i += 1;
      }
    };
}

pub fn get_avg_linedata(list: &[Vec<f64>]) -> Vec<(i32,i32)> {
  let mut lines = Vec::with_capacity(list.len());
  for (i, entry) in list.iter().enumerate() {
    lines.push((i as i32, mean(entry.as_slice())));
  }
  lines
}

fn make_linedata(list: &[f64]) -> Vec<(i64,f64)> {
  let mut lines = Vec::with_capacity(list.len());
  for (i, entry) in list.iter().enumerate() {
    lines.push((i as i64, *entry as f64));
  }
  lines
}

fn get_candlestick_data(list: &[Vec<f64>]) -> Vec<(i64, f64, f64, f64, f64)> {
  let mut res = Vec::with_capacity(list.len());
  let shortest = shortest_array_length(&list) as i64;
  for i in 0..shortest {
    let mut high: f64 = std::f64::MIN;
    let mut low: f64 = std::f64::MAX;
    for entry in list {
      let value = entry[i as usize] as f64;
      if value > high {
        high = value;
      }
      if value < low {
        low = value;
      }
    }
    res.push((i, high,high,low,low));
  }
  res
}

pub fn reward_graph_b<DB: DrawingBackend>(image_backend: &DrawingArea<DB, Shift>, rewards: &[Reward], zoom_opt: Option<f64>) {
  let zoom = zoom_opt.unwrap_or(1.0);
  let line_data = make_linedata(rewards);
  let value_max = line_data.len() as i64 - 1;
  let min_value = -501.0;
  let max_value = 1.0;
  let mut lines = Vec::new();
  lines.push(("PPO".to_owned(), line_data));
  let mut plot = make_chart(image_backend, zoom, "Reward per Episode", RangedCoordi64::from(0..value_max), RangedCoordf64::from(min_value..max_value), "Episode", "Reward");
  draw_lines!(plot, lines);
  plot.configure_series_labels()
    .position(SeriesLabelPosition::LowerMiddle)
    .label_font(("Arial", 26.0).into_font())
    .background_style(&WHITE.mix(0.8))
    .border_style(&BLACK)
  .draw().unwrap();
}

pub fn reward_graph(file_name: &str, rewards: &[Reward], zoom_opt: Option<f64>) {
  let image = image_backend(file_name, (720, 480));
  reward_graph_b(&image, rewards, zoom_opt);
}

pub fn shortest_array_length(arr: &[RewardVector]) -> usize {
  let mut shortest = std::usize::MAX;
  for rewards in arr {
    if rewards.len() < shortest {
      shortest = rewards.len();
    }
  }
  shortest
}

pub fn reward_err_graph(file_name: &str, algo_name: &str, rewards_list: &[RewardVector], zoom_opt: Option<f64>) {
  let zoom = zoom_opt.unwrap_or(1.0);
  let stroke_width = if zoom == 1.0 {1} else {(zoom * 0.6) as u32};
  let mut value_max = std::i64::MAX;
  let candles = get_candlestick_data(&rewards_list);
  for rewards in rewards_list {
    if rewards.len() as i64 - 1 < value_max {
      value_max = rewards.len() as i64 - 1;
    }
  }
  let (avg_line, min_value, max_value) = {
    let mut min_value = std::f64::MAX;
    let mut max_value = std::f64::MIN;
    let mut avg = Vec::new();
    let shortest = shortest_array_length(&rewards_list);
    let avg_amount = rewards_list.len() as f64;
    for i in 0..shortest {
      let mut total: f64 = 0.0;
      for rewards in rewards_list {
        let actual = rewards[i] as f64;
        if actual > max_value {
          max_value = actual;
        }
        if actual < min_value {
          min_value = actual;
        }
        total += actual;
      }
      let new_val = total / avg_amount;
      avg.push((i as i64, new_val));
    }
    let range = (max_value - min_value) * 0.05;
    (avg, min_value - range, max_value + range)
  };
  let image = image_backend(file_name, (720 * zoom as u32, 480 * zoom as u32));
  let mut plot = make_chart(&image, zoom, "Average Reward per Episode", RangedCoordi64::from(0..value_max), RangedCoordf64::from(min_value..max_value), "Episode", "Reward");
  let color = get_color(0);
  // let candle_stick_line_color = ShapeStyle{color: color.clone(),stroke_width,filled: true};
  plot.draw_series(
    candles.iter().map(|x| {
      // let stick = CandleStick::new(x.0, x.1, x.2, x.3, x.4, &color, &color, 5 + stroke_width);
      // stick.style = candle_stick_line_color.clone();
      CandleStick::new(x.0, x.1, x.2, x.3, x.4, &color, &color, 5 + stroke_width)
    })
  ).unwrap();
  plot.draw_series(LineSeries::new(
    avg_line,
    ShapeStyle{color,stroke_width,filled: true})).unwrap()
    .label(algo_name)
    .legend(move |(x, y)| PathElement::new(vec![(x, y + stroke_width as i32), (x + 20 + (5 * (zoom * 0.2) as i32), y + stroke_width as i32)], &get_color(0)));
  plot.configure_series_labels()
    .position(SeriesLabelPosition::LowerMiddle)
    .label_font(("Arial", 14.0 * zoom).into_font())
    .background_style(&WHITE.mix(0.8))
    .border_style(&BLACK)
  .draw().unwrap();
}
pub fn simple_line_graph(file_name: &str, rewards: RewardVector, x_label: String, y_label: String, zoom_opt: Option<i64>) {
  let zoom = zoom_opt.unwrap_or(1);
  let zoom_f = zoom as f64;
  let line_data = make_linedata(&rewards);
  let value_max = line_data.len() as i64 - 1;
  let mut lines = Vec::new();
  lines.push(("DeepQ".to_owned(), line_data));
  let image = image_backend(file_name, (720, 645));
  let min_value = -2.0;
  let max_value = 2.0;
  let mut chart = ChartBuilder::on(&image)
    .x_label_area_size(43 * zoom as u32)
    .y_label_area_size(53 * zoom as u32)
    .margin(12 * zoom as u32)
    .build_cartesian_2d(0..value_max, min_value..max_value).unwrap();
  let color = BLACK.to_rgba();
  let color2 = BLACK.to_rgba();
  let mut mesh_configurer = chart.configure_mesh();
  mesh_configurer.y_desc(y_label)
    .x_desc(x_label)
    .x_label_style(("Arial", 12.0 * zoom_f).into_font())
    .y_label_style(("Arial", 12.0 * zoom_f).into_font())
    .axis_desc_style(("Arial", 20.0 * zoom_f).into_font());
  if zoom > 1 {
    mesh_configurer.bold_line_style(ShapeStyle{color,stroke_width: ((zoom_f * 0.3) as u32),filled: true})
    .light_line_style(ShapeStyle{color: color2,stroke_width: ((zoom_f * 0.3) as u32),filled: true});
  }
  mesh_configurer.draw().unwrap();
  draw_lines!(chart, lines);
}