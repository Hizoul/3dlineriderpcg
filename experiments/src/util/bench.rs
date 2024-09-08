use std::time::Instant;
pub type BenchFunction = Box<dyn Fn() -> u128>;
pub type BenchFunctions = Vec<(String, BenchFunction)>;
pub type BenchResult = (String, Vec<(u128, u128, u128)>);
use std::time::{SystemTime, UNIX_EPOCH};

pub fn date_now() -> u128  {
  let start = SystemTime::now();
  let since_the_epoch = start
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards");
  since_the_epoch.as_millis()
}

pub fn bench_functions(repetitions: usize, save_to_opt: Option<&str>, functions_to_bench: &[(String, BenchFunction)]) -> Vec<BenchResult> {
  let time = date_now();
  let save_to = save_to_opt.unwrap_or("./");
  let file_name = format!("{}/{}_{}", save_to, time, CSV_FILE_NAME);
  let host_name = std::env::var("MACHINENAME").unwrap_or("Undeterminable".to_owned());
  let mut results = Vec::with_capacity(functions_to_bench.len());
  for (func_num, (name, function)) in functions_to_bench.iter().enumerate() {
    let full_name = format!("{}_{}", host_name, name);
    let mut sub_results = Vec::with_capacity(repetitions);
    for iteration in 0..repetitions {
      println!("Benchmark #{} {} Iteration {}/{}", func_num, name, iteration, repetitions);
      let timestamp = date_now();
      let start = Instant::now();
      let function_result = function();
      let end = Instant::now();
      let full_time = (end-start).as_nanos();
      sub_results.push((function_result, full_time, timestamp));
      append_to_file(&file_name, &format!("{},{},{},{}\n", full_name, function_result, full_time, timestamp));
    }
    results.push((name.clone(), sub_results));
  }
  results
}

pub fn csv_to_bench_results(csv: &str) -> Vec<BenchResult> {
  let mut bench_results: Vec<BenchResult> = Vec::new();
  for line in csv.split("\n") {
    let line_parts: Vec<&str> = line.split(",").collect();
    let name = line_parts[0].to_owned();
    let function_time: u128 = line_parts[1].parse().unwrap();
    let real_time: u128 = line_parts[2].parse().unwrap();
    let timestamp: u128 = line_parts[3].parse().unwrap();
    bench_results.push(
      (name, vec![(function_time, real_time, timestamp)])
    );
  }
  bench_results
}
use std::collections::HashMap;

pub fn bench_results_summarize(bench_results: &[BenchResult]) -> String {
  let mut bench_res_csv = "Name, Slowest, Fastest, Average, Median, Total#\n".to_owned();
  let mut value_map: HashMap<String, Vec<(u128, u128, u128)>> = HashMap::new();

  for (name, sub_res) in bench_results {
    if value_map.get(name).is_none() {
      value_map.insert(name.clone(), Vec::new());
    }
    let val = value_map.get_mut(name).unwrap();
    val.extend(sub_res);
  }
  for (name, sub_res) in value_map.iter() {
    let mut min = std::u128::MAX;
    let mut max = std::u128::MIN;
    let mut median: u128 = 0;
    let mut total: u128 = 0;
    let middle = sub_res.len() / 2;
    for (i, (function_time, _, _)) in sub_res.iter().enumerate() {
      total += function_time;
      if function_time < &min {min = *function_time};
      if function_time > &max {max = *function_time};
      if i == middle {median = *function_time};
    }
    bench_res_csv.push_str(&format!("{},{},{},{},{}\n", name, max, min, (total as usize / sub_res.len()), median));
  }
  bench_res_csv
}


pub fn bench_results_to_csv(bench_results: &[BenchResult]) -> (String, String) {
  let mut bench_csv = "".to_owned();//"Name, Function Time, Full Time, TimeStamp\n".to_owned();
  let mut bench_res_csv = "Name, Slowest, Fastest, Average, Median\n".to_owned();
  let host_name = std::env::var("HOSTNAME").unwrap_or("Undeterminable".to_owned());
  for (name, sub_res) in bench_results {
    let mut min_val = std::u128::MAX;
    let mut max_val = std::u128::MIN;
    let mut median: u128 = 0;
    let mut total: u128 = 0;
    let middle = sub_res.len() / 2;
    for (i, (function_time, full_time, time_stamp)) in sub_res.iter().enumerate() {
      total += function_time;
      if function_time < &min_val {min_val = *function_time};
      if function_time > &max_val {max_val = *function_time};
      if i == middle {median = *function_time};
      bench_csv.push_str(&format!("{},{},{},{}\n", format!("{}_{}", host_name, name), function_time, full_time, time_stamp));
    }
    bench_res_csv.push_str(&format!("{},{},{},{},{}\n", format!("{}_{}", host_name, name), max_val, min_val, (total as usize / sub_res.len()), median));
  }
  (bench_csv, bench_res_csv)
}

pub fn append_to_file(path: &str, content: &str) {
  use std::fs::OpenOptions;
  use std::io::Write;
  let mut file = OpenOptions::new()
  .write(true)
  .append(true)
  .create(true)
  .open(path)
  .unwrap();
  file.write_all(content.as_bytes()).unwrap();
}

const CSV_FILE_NAME: &str = "bench_times.csv";

pub fn bench_functions_and_save(repetitions: usize, save_to: &str, functions_to_bench: &[(String, BenchFunction)]) {
  let results = bench_functions(repetitions, Some(save_to), functions_to_bench);
  let (bench_csv, _) = bench_results_to_csv(&results);
  let time = date_now();
  append_to_file(&format!("{}/{}_{}", save_to, time, CSV_FILE_NAME), &bench_csv);
}

pub fn merge_csv_files(path: &str) -> String {
  use walkdir::WalkDir;
  let mut full_csv = String::new();
  for entry_opt in WalkDir::new(path) {
    if let Ok(entry) = entry_opt {
      let file_name = entry.path().to_str().unwrap();
      if file_name.ends_with(CSV_FILE_NAME) {
        full_csv.push_str(&std::fs::read_to_string(file_name).unwrap());
      }
    }
  }
  full_csv
}

pub fn merge_csv_files_to(path: &str, result_file: &str) {
  append_to_file(result_file, &merge_csv_files(path));
}


#[cfg(test)]
pub mod test {
  use super::bench_functions;
  #[test]
  pub fn test_bench_function(){
    println!("GOT TEST");
    bench_functions(200, None, &[
       ("testfunction".to_owned(), Box::new(|| {
         1
       }))
    ]);
  }
}