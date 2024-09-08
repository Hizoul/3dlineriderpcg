pub mod util;
pub use util::bench::{
  date_now, bench_functions_and_save, merge_csv_files_to,
  csv_to_bench_results, bench_results_summarize
};
pub mod experiments;
pub use std::env::current_dir;
use clap::{Arg, Command};

fn main() {
  let mut cli_app = Command::new("Rusty RL Experiments Runner")
  .version("1.0.0")
  .author("Matthias Müller-Brockhausen <git@mmb2.click>")
  .about("Command Line Utility to run RL Experiments and process results")
  .arg(
    Arg::new("csv_dir").short('m').long("merge").help("Specify a directory that is scanned for 'bench_times.csv' files and merges them into one.")
  )
  .arg(
    Arg::new("csv_file_to_summarize").short('s').long("summarize").help("Summarizes the specified file into a result.csv. Result is written to 'folder' argument or CWD")
  ).arg(
    Arg::new("experiment_name").short('e').long("experiment").help("Specifiy the name of the experiment that should be run"))
  .arg(
    Arg::new("folder").short('r').long("results").help("Specifiy the folder in which to save the result CSV files. Default: Current working directory"));
  let matches = cli_app.clone().get_matches();
  
  if let Some(merge_path) = matches.get_one::<String>("csv_dir") {
    println!("About to merge CSV result files in directory: {}", merge_path);
    merge_csv_files_to(merge_path, &format!("{}/merged.csv", merge_path))
  } else if let Some(file_to_summarize) = matches.get_one::<String>("csv_file_to_summarize") {
    let file = std::fs::read_to_string(file_to_summarize).unwrap();
    let bench_res = csv_to_bench_results(&file);
    let bench_summary = bench_results_summarize(&bench_res);
    let cwd_buf = current_dir().unwrap();
    let cwd = cwd_buf.to_str().unwrap();
    let cwd_string = cwd.to_owned();
    let results_folder = matches.get_one::<String>("folder").unwrap_or(&cwd_string);
    std::fs::write(format!("{}/{}_summary.csv", results_folder, date_now()), bench_summary).unwrap();
  } else if let Some(experiment_name) = matches.get_one::<String>("experiment_name") {
    println!("About to run experiments: {}", experiment_name);
    let cwd_buf = current_dir().unwrap();
    let cwd = cwd_buf.to_str().unwrap();
    let cwd_string = cwd.to_owned();
    let _results_folder = matches.get_one::<String>("folder").unwrap_or(&cwd_string);
    match experiment_name.clone().as_str() {
      #[cfg(feature = "lrpcg")]
      "linerider_heu_log" => {
        crate::experiments::linerider::heuristic_log::make_heuristic_logs();
      },
      #[cfg(feature = "lrpcg")]
      "linerider_booster_optim" => {
        crate::experiments::linerider::booster_strength_exp::find_optimal_booster_strength();
      },
      #[cfg(feature = "lrpcg")]
      "linerider_extract" => {
        crate::experiments::linerider::post_analysis::extract_linerider_successes(&matches.get_one::<String>("folder"));
      },
      _ => {println!("Experiment \"{}\" is not known to me. It might be disabled due to a feature flag.", experiment_name)}
    }
  } else {
    cli_app.print_help().unwrap();
  }  
}