use rusty_gym::{load_run_convert_python, GymEnvironment, ReplayableGymEnvironment, RunData};

use crate::env::LineRider3DEnv;

use super::{is_freepoint_actionspace, LineRiderSim};

pub fn replay_single_episode(file_name: &str, episode: usize) {
  let mut run: RunData = load_run_convert_python(file_name);
  if run.episodes.len() < episode {
    panic!("Cant load episode {} as only {} are available", episode, run.episodes.len());
  }
  let episode_data = run.episodes.index(episode);
  let sim = LineRiderSim::default_with_ui();
  let mut env = LineRider3DEnv::new(sim, None);
  env.load_config(&run.env_config);
  env.sim.add_debug_cam();
  env.skip_simulation = true;
  env.use_seed(episode_data.seed);
  env.reset();
  println!("LOG IS {:?}", episode_data.log);
  for action in &episode_data.log {
    let mut action_to_use = action.clone();
    if action_to_use[3] == 0.0 {
      action_to_use[3] = 1.0;
    }
    env.step(&action_to_use);
  }
  println!("line now is {:?}", env.lines);
  if is_freepoint_actionspace(env.sim.config.action_type) {
    env.add_lines_freeroam();
  } else {
    env.add_lines();
  }
  env.sim.app.run();
}
pub fn replay_single_episode_cli() {
  let args: Vec<String> = std::env::args().collect();
  let file_name: String = args[1].parse().expect("First Arg is filename");
  let episode: usize = args[2].parse().expect("secpmd Arg is episode number");
  replay_single_episode(&file_name, episode);
}

// "./trl-experiments/LineRider3D-Env-v0/heu/highlights_chk_down.tlr",
// 20