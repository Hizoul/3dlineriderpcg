#[cfg(feature = "vis")]
pub mod plot;
use rusty_gym::{
  algo::SelfTrainingAlgo, EnrichedEpisodeData, RewardVector,
  RlExperimentHelper, RunData, RunDataEnriched, load_run_convert_python,
  EnvironmentMaker, sum
};
use xp_tools::fs::save_cbor_and_flate_to_path;
use compressed_vec::CompressedVec;


#[cfg(feature = "vis")]
use crate::eval::plot::reward_err_graph;



#[cfg(not(target_arch = "wasm32"))]
pub fn read_replays_and_enrich_them(result_dir: &str, evaluator: &mut RlExperimentHelper) {
  use rusty_gym::Reward;
  use walkdir::WalkDir;
  use rayon::prelude::*;
  for entry_opt in WalkDir::new(result_dir) {

    if let Ok(entry) = entry_opt {
      let file_name = entry.path().to_str().unwrap();
      println!("Checking relevance {:?}", file_name);
      if file_name.ends_with(".tlr") {
        if std::path::Path::new(&format!("{}x", file_name)).exists() {
          println!("SKipping because already enriched {:?}", file_name);

        } else {
          
          println!("LOADING FILE {:?}", file_name);
          let mut run: RunData = load_run_convert_python(entry.path().to_str().unwrap());
          let mut enriched_episodes: CompressedVec<EnrichedEpisodeData> = CompressedVec::new();
          let mut rpe: Vec<Reward> = Vec::with_capacity(run.episodes.len());
          println!("Sending all episodes to threadpool");
          let episodes = run.episodes.clone_to_vec();
          let env_init = evaluator.environments.iter_mut().find(|env_to_check| {
            let env = (*env_to_check)(&run.env_config);
            env.get_name() == run.env
          }).unwrap();
          let config = run.env_config.clone();
          println!("For {} episodes Chunk size is {} rayon allows {}", episodes.len(), (episodes.len() / rayon::current_num_threads()) /2, rayon::current_num_threads());
          let collected: Vec<Vec<EnrichedEpisodeData>> = episodes.par_chunks((episodes.len() / rayon::current_num_threads()) /2).map(|sub_episodes| {
            let mut enriched_chunk: Vec<EnrichedEpisodeData> = Vec::with_capacity(sub_episodes.len());
            let mut env = env_init(&config);
            for episode in 0..sub_episodes.len() {
              let episode_data = &sub_episodes[episode];
              if let Some(env_config) = &episode_data.env_params {
                env.load_config(env_config);
              }
              env.use_seed(episode_data.seed);
              env.reset();
              let mut rewards: RewardVector = Vec::with_capacity(episode_data.log.len());
              for log_entry in episode_data.log.iter() {
                let step = env.step(&log_entry);
                rewards.push(step.reward);
              }
              let enriched_episode = EnrichedEpisodeData {
                log: episode_data.log.clone(),
                seed: episode_data.seed,
                algorithm_hyperparams: episode_data.algorithm_hyperparams.clone(),
                env_params: episode_data.env_params.clone(),
                rewards,
                task_completion: 0.0, episode_nr: episode, additional_info: None
              };
              enriched_chunk.push(enriched_episode);
            }
            enriched_chunk
          }).collect();
          println!("Reorganizing collected episodes");
          let mut sorted_enriched: Vec<EnrichedEpisodeData> = Vec::with_capacity(run.episodes.len());
          for sub_episodes in collected {
            for sub_episode in sub_episodes {
              sorted_enriched.push(sub_episode);
            }
          }
          println!("Sorting collected episodes");
          sorted_enriched.sort_by(|a, b| a.episode_nr.cmp(&b.episode_nr));
          for enriched_episode in sorted_enriched {
            rpe.push(sum(&enriched_episode.rewards));
            enriched_episodes.push(enriched_episode);
          }

          run.reward_per_episode = Some(rpe);
          let algo_dir = format!("{}/{}/{}", result_dir, run.env, run.algo);
          save_cbor_and_flate_to_path(format!("{}/{}.tlrx", algo_dir, run.uid).as_str(), &RunDataEnriched::new(run.clone(), enriched_episodes.clone()));
          save_cbor_and_flate_to_path(entry.path().to_str().unwrap(), &run);
        }
      }
    }
  }
}


pub fn enrich_runs(result_dir: &str, envs: Vec<EnvironmentMaker>) {
  println!("Looking for runs to enrich in {}", result_dir);
  let algorithms: Vec<Box<dyn SelfTrainingAlgo>> = vec![]; //vec![Box::new(a2c_sb3_algo), Box::new(dqn_sb3_algo)];

  let mut evaluator = RlExperimentHelper::with_config(result_dir.to_owned(), algorithms, envs);
  read_replays_and_enrich_them(&result_dir, &mut evaluator);
}
