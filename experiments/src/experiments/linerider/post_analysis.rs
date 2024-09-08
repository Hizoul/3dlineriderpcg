
use linerider::{env::LineRider3DEnv, util::consts::*};
use rusty_gym::{ReplayableGymEnvironment,
  EnrichedEpisodeData, RewardVector, GymEnvironment,
  RunData, RunDataEnriched, load_run_convert_python, sum};
use xp_tools::fs::save_cbor_and_flate_to_path;
use compressed_vec::CompressedVec;
use std::collections::HashMap;


pub fn extract_linerider_successes(result_dir_opt: &Option<&String>) {
  read_replays_and_extract_success(result_dir_opt)
}


#[cfg(not(target_arch = "wasm32"))]
pub fn read_replays_and_extract_success(result_dir_opt: &Option<&String>) {
  use rusty_gym::Reward;
  use walkdir::{WalkDir, DirEntry};
  use rayon::prelude::*;
  use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

  let m = MultiProgress::new();
  let sty = ProgressStyle::with_template(
      "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
  )
  .unwrap()
  .progress_chars("##-");
  m.println("starting!").unwrap();


  let a = "./trl-experiments".to_owned();
  let result_dir = result_dir_opt.unwrap_or(&a);
  let v: Vec<Result<DirEntry, walkdir::Error>> = WalkDir::new(result_dir).into_iter().collect();
  v.par_iter().for_each(|entry_opt| {
    if let Ok(entry) = entry_opt {
      let file_name = entry.path().to_str().unwrap();
      m.println(format!("Checking relevance {:?}", file_name)).unwrap();
      if file_name.ends_with(".tlr") && !file_name.contains("highlights_") {
        if std::path::Path::new(&format!("{}x", file_name)).exists() {
          m.println(format!("SKipping because already enriched {:?}", file_name)).unwrap();
        } else {
          m.println(format!("LOADING FILE {:?}", file_name)).unwrap();
          let mut success_track = 0;
          let mut success_ball = 0;
          let mut success_both = 0;
          let mut run: RunData = load_run_convert_python(entry.path().to_str().unwrap());
          let mut enriched_episodes: CompressedVec<EnrichedEpisodeData> = CompressedVec::new();
          let mut rpe: Vec<Reward> = Vec::with_capacity(run.episodes.len());
          let mut rpe_all: Vec<Reward> = Vec::with_capacity(run.episodes.len());
          m.println(format!("Sending all episodes to threadpool")).unwrap();
          let episodes = run.episodes.clone_to_vec();
          let pb = m.add(ProgressBar::new(episodes.len() as u64));
          pb.set_style(sty.clone());
          let env_init = |config: &HashMap<String, String>| {
            let mut env = LineRider3DEnv::new(linerider::simulator::LineRiderSim::new(false), None);
            env.load_config(config);
            env
          };
          let config = run.env_config.clone();
          let mut threads_to_use = (episodes.len() / rayon::current_num_threads()) /2;
          if threads_to_use == 0 {
            threads_to_use = 2;
          }
          m.println(format!("For {} episodes Chunk size is {} rayon allows {}", episodes.len(), threads_to_use, rayon::current_num_threads())).unwrap();
          
          let collected: Vec<(Vec<EnrichedEpisodeData>, Vec<Option<EnrichedEpisodeData>>, Vec<Option<EnrichedEpisodeData>>)> = episodes.par_chunks(threads_to_use).map(|sub_episodes| {
            let mut enriched_chunk: Vec<Option<EnrichedEpisodeData>> = Vec::with_capacity(sub_episodes.len());
            let mut unsuccessful_episode_chunk: Vec<Option<EnrichedEpisodeData>> = Vec::with_capacity(sub_episodes.len());
            let mut all_episodes: Vec<EnrichedEpisodeData> = Vec::with_capacity(sub_episodes.len());
            let mut env = env_init(&config);
            for episode in 0..sub_episodes.len() {
              let episode_data = &sub_episodes[episode];
              if let Some(env_config) = &episode_data.env_params {
                env.load_config(env_config);
              }
              env.skip_simulation = true;
              env.use_seed(episode_data.seed);
              env.reset();
              let mut rewards: RewardVector = Vec::with_capacity(episode_data.log.len());
              for log_entry in episode_data.log.iter() {
                let step = env.step(&log_entry);
                rewards.push(step.reward);
              }
              rewards.pop();
              env.skip_simulation = false;
              match env.sim.config.action_type {
                ACTION_TYPE_FREE_POINTS | ACTION_TYPE_FREE_POINTS_WITH_TP |
                  ACTION_TYPE_FREE_POINTS_RELATIVE | ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE => {
                    env.add_lines_freeroam();
                }
                _ => {
                  env.add_lines();
                }
              };
              let sim_res = env.sim.simulate_till_end(env.sim.config.simulation_steps);
              let reward = env.get_reward_from_simulation_result(sim_res.clone());
              let task_completion = if sim_res.goal_reached && env.track_reached_goal {
                3.0
              } else if sim_res.goal_reached && !env.track_reached_goal {
                2.0
              } else if !sim_res.goal_reached && env.track_reached_goal {
                1.0
              } else {0.0};
              rewards.push(reward);
              let enriched_episode = EnrichedEpisodeData {
                log: episode_data.log.clone(),
                seed: episode_data.seed,
                rewards, algorithm_hyperparams: episode_data.algorithm_hyperparams.clone(),
                env_params: episode_data.env_params.clone(),
                task_completion, episode_nr: episode, additional_info: Some(sim_res.to_map())
              };
              all_episodes.push(enriched_episode.clone());
              let to_push: Option<EnrichedEpisodeData> = if task_completion >= 1.0 {Some(enriched_episode.clone())} else {None};
              enriched_chunk.push(to_push);
              unsuccessful_episode_chunk.push(if task_completion <= 2.0 {Some(enriched_episode)} else {None});
              
              pb.inc(1);
            }
            (all_episodes, enriched_chunk, unsuccessful_episode_chunk)
          }).collect();
          m.println(format!("Reorganizing collected episodes")).unwrap();
          let mut sorted_enriched: Vec<EnrichedEpisodeData> = Vec::with_capacity(run.episodes.len());
          let mut sorted_unsuccessful_enriched: Vec<EnrichedEpisodeData> = Vec::with_capacity(run.episodes.len());
          let mut all_episodes: CompressedVec<EnrichedEpisodeData> = CompressedVec::new();
          for sub_episodes in collected {
            for sub_episode in sub_episodes.0 {
              rpe_all.push(sum(&sub_episode.rewards));
              all_episodes.push(sub_episode)
            }
            for sub_episode in sub_episodes.1 {
              if let Some(actual_episode) = sub_episode {
                if actual_episode.task_completion == 1.0 || actual_episode.task_completion == 3.0 {
                  success_track += 1;
                }
                if actual_episode.task_completion >= 2.0 {
                  success_ball += 1;
                }
                if actual_episode.task_completion >= 3.0 {
                  success_both += 1;
                }
                sorted_enriched.push(actual_episode);
              }
            }
            for sub_episode in sub_episodes.2 {
              if let Some(actual_episode) = sub_episode {
                sorted_unsuccessful_enriched.push(actual_episode);
              }
            }
          }
          m.println(format!("Sorting collected episodes {}", sorted_unsuccessful_enriched.len())).unwrap();
          sorted_enriched.sort_by(|a, b| a.episode_nr.cmp(&b.episode_nr));
          sorted_unsuccessful_enriched.sort_by(|a, b| a.episode_nr.cmp(&b.episode_nr));
          for enriched_episode in &sorted_enriched {
            rpe.push(sum(&enriched_episode.rewards));
            enriched_episodes.push(enriched_episode.clone());
          }
          
          let algo_dir = format!("{}/{}/{}", result_dir, run.env, run.algo);
          println!("ATTEMPTING TO SAVE AT{:?}", algo_dir);
          run.reward_per_episode = Some(rpe_all);
          all_episodes.finalize();
          let mut to_save = all_episodes.clone();
          save_cbor_and_flate_to_path(format!("{}/{}.tlrx", result_dir, run.uid).as_str(), &RunDataEnriched::new(run.clone(), to_save));
          enriched_episodes.finalize();
          to_save = enriched_episodes.clone();
          run.reward_per_episode = Some(rpe);
          save_cbor_and_flate_to_path(format!("{}/highlights_{}.tlrx", result_dir, run.uid).as_str(), &RunDataEnriched::new(run.clone(), to_save));
          
          let mut new_run = run.clone();
          new_run.episodes.clear();
          for enriched_episode in sorted_enriched {
            new_run.episodes.push(enriched_episode.to_regular_episode());
          }
          new_run.episodes.finalize();
          new_run.uid = format!("highlights_{}", run.uid);
          m.println(format!("Run has {} ({}%) Tracks that reach goal and {} ({}%) Balls that reach goal {} ({}%) track+ball reaches total is {} is {}", success_track, (success_track as f32/ run.episodes.len() as f32) * 100.0, success_ball, (success_ball as f32 / run.episodes.len() as f32) * 100.0, success_both, (success_both as f32 / run.episodes.len() as f32) * 100.0, run.episodes.len(), run.uid)).unwrap();
          save_cbor_and_flate_to_path(format!("{}/{}.tlr", result_dir, new_run.uid).as_str(), &new_run);
          let mut new_run_unsuccessful = run.clone();
          new_run_unsuccessful.episodes.clear();
          for enriched_episode in sorted_unsuccessful_enriched {
            new_run_unsuccessful.episodes.push(enriched_episode.to_regular_episode());
          }
          new_run_unsuccessful.episodes.finalize();
          new_run_unsuccessful.uid = format!("anti_highlights_{}", run.uid);
          save_cbor_and_flate_to_path(format!("{}/{}.tlr", result_dir, new_run_unsuccessful.uid).as_str(), &new_run_unsuccessful);
        }
      }
    }
  });
  m.clear().unwrap();
}

