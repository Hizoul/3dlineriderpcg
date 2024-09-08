use bevy::{prelude::ResMut, ecs::schedule::NextState};
use bevy_egui::egui::{
  Button, Context, Color32, Ui,
  emath::{pos2, Rect, Pos2}, epaint::Shape
};
use std::{sync::{Arc, RwLock, Mutex, atomic::{Ordering, AtomicUsize}}, collections::HashMap};
use rusty_gym::{GymEnvironment, ReplayableGymEnvironment, episode_to_reward_vec_v_rep, sum, eval::{RunData, plot::reward_graph_b, async_load_run_convert_python}, EpisodeData};
use crate::{
  try_read, try_write,
  replay::plot::{EguiBackend, EguiPlottersPixels, EguiPlottersTexts, translate_and_paint_cmds_to_area},
  env::LineRider3DEnv, simulator::{system::TrackToAdd, GameState, config::is_freepoint_actionspace}
};
use plotters::prelude::*;

use super::LineRiderConfig;

pub struct EguiRunDetail {
  pub state: Arc<RwLock<EguiRunDetailState>>
}

pub type ReplayData = (EpisodeData, String, HashMap<String, String>);
pub struct EguiEpisodeRenderState {
  pub to_replay: Option<ReplayData>,
  pub current_tick: AtomicUsize,
  pub current_episode: AtomicUsize,
  pub paused: bool,
  pub ms_since_last_tick: usize
}

impl EguiEpisodeRenderState {
  pub fn new(to_replay: Option<ReplayData>) -> EguiEpisodeRenderState {
    EguiEpisodeRenderState {
      to_replay,
      current_tick: AtomicUsize::new(0),
      current_episode: AtomicUsize::new(0),
      paused: true, // TODO: only true for web
      ms_since_last_tick: 0
    }
  }
}

pub type LockedEguiEpisodeRenderState = Arc<RwLock<EguiEpisodeRenderState>>;
pub struct EguiRunDetailState {
  dir_name: String,
  pub use_ipfs: bool,
  pub displayed_run: Option<String>,
  pub calculated_avg_rewards: bool,
  pub loaded_run: Option<RunData>,
  pub is_loading: bool,
  displayed_episode_state: LockedEguiEpisodeRenderState,
  ctx: Option<Context>,
  rendered_graph_avg: Option<(EguiPlottersPixels, EguiPlottersTexts)>,
  rendered_graph: Option<(EguiPlottersPixels, EguiPlottersTexts)>,
  pub last_rendered_episode: usize
}

pub type LockedEguiRunDetailState = Arc<RwLock<EguiRunDetailState>>;

impl EguiRunDetailState {
  pub fn new(dir_name: String, use_ipfs: bool, displayed_episode_state: LockedEguiEpisodeRenderState) -> EguiRunDetailState {
    EguiRunDetailState {
      dir_name,
      displayed_run: None,
      loaded_run: None,
      use_ipfs,
      calculated_avg_rewards: false,
      is_loading: false,
      displayed_episode_state,
      ctx: None,
      rendered_graph_avg: None,
      rendered_graph: None,
      last_rendered_episode: usize::MAX
    }
  }
}

pub const GRAPH_OFFSET_LEFT: usize = 63;
pub const GRAPH_OFFSET_RIGHT: usize = 10;

pub fn get_episode_index(max_number: usize, rect: &Rect, click: &Pos2, plot_tick_amount: usize) -> Option<usize> {
  if (rect.min.x < click.x && rect.max.x > click.x) &&
    (rect.min.y < click.y && rect.max.y > click.y) {
    let total_width = (rect.max.x - rect.min.x) as usize;
    let grid_length = total_width - GRAPH_OFFSET_LEFT - GRAPH_OFFSET_RIGHT;
    let width_per_episode = (grid_length as f64 / max_number as f64) + plot_tick_amount as f64;
    let relative_click = pos2(click.x - rect.min.x, click.y - rect.min.y);
    let grid_pos = relative_click.x as usize - GRAPH_OFFSET_LEFT;
    let entry_clicked_on = (grid_pos as f64 / width_per_episode) as usize;
    Some(entry_clicked_on)
  } else {
    None
  }
}

pub fn episode_to_pos(max_number: usize, rect: &Rect, episode: usize, plot_tick_amount: usize) -> usize {
  let total_width = (rect.max.x - rect.min.x) as f64;
  let grid_length = total_width - GRAPH_OFFSET_LEFT as f64 - GRAPH_OFFSET_RIGHT as f64;
  let width_per_episode = (grid_length / max_number as f64) + plot_tick_amount as f64;
  (episode as f64 * width_per_episode) as usize + GRAPH_OFFSET_LEFT
}

impl EguiRunDetail {
  pub fn new(dir_name: String, use_ipfs: bool) -> EguiRunDetail {
    let state = Arc::new(RwLock::new(EguiRunDetailState::new(dir_name, use_ipfs, Arc::new(RwLock::new(EguiEpisodeRenderState::new(None))))));
    EguiRunDetail {
      state
    }
  }
  pub fn set_displayed_run(&mut self, displayed_run: Option<String>) {
    let mut state = try_write!(self.state);
    state.displayed_run = displayed_run;
    state.calculated_avg_rewards = false;
  }

  pub fn update_state_after_loading_run(l_state: LockedEguiRunDetailState) {
    let mut state = try_write!(l_state);
    let mut run_data = state.loaded_run.as_ref().unwrap().clone();
    state.rendered_graph_avg = None;
    state.rendered_graph = None;
    state.calculated_avg_rewards = false;
    state.last_rendered_episode = std::usize::MAX;
    state.is_loading = false;
    if run_data.episodes.len() > 0 {
      let mut episode_state = try_write!(state.displayed_episode_state);
      let episode = run_data.episodes.index(0).clone(); // todo: use correct env config. also check for reward graph
      episode_state.to_replay = Some((episode.clone(), run_data.env.clone(), run_data.env_config.clone()));
    } else {
      let mut episode_state = try_write!(state.displayed_episode_state);
      episode_state.to_replay = None;
    }
  }

  pub async fn load_run(l_state: LockedEguiRunDetailState) {
    let needs_loading = {
      let state = try_read!(l_state);
      if state.displayed_run.is_some() && state.loaded_run.is_none() {
        !state.is_loading 
      } else if state.loaded_run.is_some() && state.displayed_run.is_some() {
        let loaded = state.loaded_run.as_ref().unwrap();
        let displayed = state.displayed_run.as_ref().unwrap();
        let shortened = displayed[0..displayed.len()-4].to_owned();
        !shortened.eq(&loaded.uid)
      } else {
        false
      }
    };
    let is_loading = {
      let state = try_read!(l_state);
      state.is_loading
    };
    if needs_loading && !is_loading {
      let (dir_name, file_name, use_ipfs) = {
        let mut state = try_write!(l_state);
        state.is_loading = true;
        (state.dir_name.clone(), state.displayed_run.as_ref().unwrap().clone(), state.use_ipfs)
      };
      let run_data: RunData = if use_ipfs {
        println!("LOADING FROM {}", format!("{}", file_name));
        let actual_file_data = xp_tools::read_url(&file_name).await;
        let mut the_run: RunData = xp_tools::load_cbor_and_flate_from_vec(actual_file_data);
        the_run.uid = file_name.clone();
        the_run
      } else {
        println!("LOADING FROM {}", format!("{}/{}", dir_name, file_name));
        async_load_run_convert_python(format!("{}/{}", dir_name, file_name).as_str()).await
      };
      println!("DONE LOADING!");
      {
        let mut state = try_write!(l_state);
        state.loaded_run = Some(run_data.clone());
      }
      EguiRunDetail::update_state_after_loading_run(l_state.clone());
      {
        let state = try_read!(l_state);
        if let Some(ctx) = state.ctx.as_ref() {
          ctx.request_repaint();
        }
      }
    }
    let mut state = try_write!(l_state);
    if !state.is_loading && state.loaded_run.is_some() {
      // let mut run_data = {
      //   state.loaded_run.as_ref().unwrap().clone()
      // };
      // let mut env = LineRider3DEnv::default();
      // env.load_config(&run_data.env_config);
      // let mut boxed_env:  Box<dyn ReplayableGymEnvironment> = Box::new(env);
      // if !state.calculated_avg_rewards && run_data.reward_per_episode.is_some() {
      //   let avg_rewards =
      //   if run_data.reward_per_episode.is_some() {
      //     run_data.reward_per_episode.expect("issome")
      //   } else {
      //     let mut avg_rewards = Vec::with_capacity(run_data.episodes.len());
      //     for ep_idx in 0..run_data.episodes.len() {
      //       let ep = run_data.episodes.index(ep_idx);
      //       let rewards = episode_to_reward_vec_v_rep(&mut boxed_env, ep);
      //       avg_rewards.push(sum(&rewards));
      //       // avg_rewards.push(1.0);
      //     }
      //     avg_rewards
      //   };
      //   let raster_size: (u32, u32) = (480, 320);
      //   let pixels_avg = Arc::new(Mutex::new(Vec::with_capacity((raster_size.0 * raster_size.1) as usize)));
      //   let texts_avg = Arc::new(Mutex::new(Vec::with_capacity((raster_size.0 * raster_size.1) as usize)));
      //   let backend = EguiBackend::new(raster_size, pixels_avg.clone(), texts_avg.clone());
      //   let drawing_area = backend.into_drawing_area();
      //   drawing_area.fill(&WHITE).unwrap();
      //   reward_graph_b(&drawing_area, &avg_rewards, Some(1.0));
      //   drawing_area.present().unwrap();
      //   {
      //     let p = pixels_avg.lock().unwrap();
      //     let t = texts_avg.lock().unwrap();
      //     state.rendered_graph_avg = Some((p.clone(), t.clone()));
      //     state.calculated_avg_rewards = true;
      //   }
      // }
      // let currently_selected_episode = {
      //   let renderer_state = try_read!(state.displayed_episode_state);
      //   renderer_state.current_episode.load(Ordering::Relaxed)
      // };
      // if state.last_rendered_episode != currently_selected_episode && run_data.episodes.len() > currently_selected_episode {
      //   state.last_rendered_episode = currently_selected_episode;
        
      //   let rewards = episode_to_reward_vec_v_rep(&mut boxed_env, run_data.episodes.index(currently_selected_episode));
      //   let raster_size: (u32, u32) = (480, 320);
      //   let pixels = Arc::new(Mutex::new(Vec::with_capacity((raster_size.0 * raster_size.1) as usize)));
      //   let texts = Arc::new(Mutex::new(Vec::with_capacity((raster_size.0 * raster_size.1) as usize)));
      //   let backend = EguiBackend::new(raster_size, pixels.clone(), texts.clone());
      //   let drawing_area = backend.into_drawing_area();
      //   drawing_area.fill(&WHITE).unwrap();
      //   reward_graph_b(&drawing_area, &rewards, Some(1.0));
      //   drawing_area.present().unwrap();
      //   {
      //     let p = pixels.lock().unwrap();
      //     let t = texts.lock().unwrap();
      //     state.rendered_graph = Some((p.clone(), t.clone()));
      //   }
      // }
    }
  }

  pub fn check_active_run(&self) {
    let state = self.state.clone();
    #[cfg(target_arch = "wasm32")] {
      wasm_bindgen_futures::spawn_local(EguiRunDetail::load_run(state));
    }
    #[cfg(not(target_arch = "wasm32"))] {
      // todo: use a global threadpool

      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async {
        let inner_runtime = tokio::runtime::Runtime::new().unwrap();
        EguiRunDetail::load_run(state).await;
        inner_runtime.shutdown_background();
      });
    }
  }

  pub fn ui(&mut self, ui: &mut Ui, mut nextstate: ResMut<NextState<GameState>>, mut track_to_add: ResMut<TrackToAdd>, mut config: ResMut<LineRiderConfig>) {
    {
      let mut state = try_write!(self.state);
      if state.ctx.is_none() {
        state.ctx = Some(ui.ctx().clone());
      }
    }
    self.check_active_run();

    let state_ref = self.state.clone();
    let mut graph_rect = Rect::NOTHING;
    {
      let has_run = {
        let state = try_read!(state_ref);
        state.displayed_run.is_some()
      };
      if has_run {
        let back_button = Button::new("Back to List");
        let back_button_resp = ui.add(back_button);
        if back_button_resp.clicked() {
          self.set_displayed_run(None);
        }
        let mut state = try_write!(state_ref);
        let _resp = ui.label(format!("Selected {:?}", state.displayed_run));
        if state.is_loading {
          ui.label(format!("Loading run, please wait."));
        }
        let mut overwrite_displayed_episode = None;
        let episode_amount = if let Some(run_data) = state.loaded_run.as_mut() {
          if let Some(reuses) = &run_data.reuses {
            let continue_button = Button::new("Transferred from");
            // todo: make heading .text_style(egui::TextStyle::Heading);
            let cb_resp = ui.add(continue_button);
            if cb_resp.clicked() {
              overwrite_displayed_episode = Some(format!("{}.tlr", reuses)); // todo: check ipfs functionality
            }
          }
          if let Some(eval_of) = &run_data.is_eval_of {
            let evalbtn = Button::new("Original Training");
            let eb_resp = ui.add(evalbtn);
            if eb_resp.clicked() {
              overwrite_displayed_episode = Some(format!("{}.tlr", eval_of)); // todo: check ipfs functionality
            }
          }
          run_data.episodes.len()
        } else {0};
        if let Some(new_run_id) = overwrite_displayed_episode {
          state.displayed_run = Some(new_run_id.clone());
          state.calculated_avg_rewards = false;
        }
        if let Some(graph) = state.rendered_graph_avg.as_ref() {
          graph_rect = translate_and_paint_cmds_to_area(ui, (480, 320), &graph.0, &graph.1);
          let episode_pos_in_graph = episode_to_pos(episode_amount, &graph_rect, state.last_rendered_episode, 0);
          let rect_should_be_at = Rect::from_min_max(pos2(0.0 + graph_rect.min.x + episode_pos_in_graph as f32, graph_rect.min.y), pos2(5.0 + graph_rect.min.x + episode_pos_in_graph as f32, graph_rect.max.y));
          let position_on_graph = Shape::rect_filled(rect_should_be_at, 0.0, Color32::from_rgba_premultiplied(0, 255, 0, 255));
          ui.painter().add(position_on_graph);
        } else {
          let slider = bevy_egui::egui::Slider::from_get_set(0.0..=episode_amount as f64, |a| {
            if a.is_some() {
              let new_index = a.unwrap() as usize;
              let to_replay = {
                let run_data = state.loaded_run.as_mut().unwrap();
                if run_data.episodes.len() > new_index {
                  Some((run_data.episodes.index(new_index).clone(), run_data.env.clone(), run_data.env_config.clone()))
                } else { None }
              };
              if let Some(replay_data) = to_replay {
                
                nextstate.set(GameState::ChooseTraining);
                let data = replay_data.0;
                let mut env = LineRider3DEnv::default();
                env.load_config(&replay_data.2);
                env.use_seed(data.seed);
                env.skip_simulation = true;
                env.reset();
                for action in &data.log {
                  env.step(action);
                }
                track_to_add.0 = env.lines.clone();
                track_to_add.1 = is_freepoint_actionspace(env.sim.config.action_type);
                track_to_add.2 = env.sim.config.goal_pos;

                config.copy_from(&env.sim.config);

                let renderer_state = try_read!(state.displayed_episode_state);
                renderer_state.current_episode.store(new_index, Ordering::Relaxed);
                renderer_state.current_tick.store(0, Ordering::Relaxed);
              }
              a.unwrap()
            } else {
              let renderer_state = try_read!(state.displayed_episode_state);
              let e = renderer_state.current_episode.load(Ordering::Relaxed);
              e as f64
            }});
          ui.add(slider);
        }
        if let Some(graph) = state.rendered_graph.as_ref() {
          let sub_rect = translate_and_paint_cmds_to_area(ui, (480, 320), &graph.0, &graph.1);
          let (current_tick, max_len) = {
            let sub_state = try_read!(state.displayed_episode_state);
            let max_len = sub_state.to_replay.as_ref().unwrap().0.log.len();
            (sub_state.current_tick.load(Ordering::Relaxed), max_len)
          };
          let episode_pos_in_graph = episode_to_pos(max_len, &sub_rect, current_tick, 0);
          let rect_should_be_at = Rect::from_min_max(pos2(0.0 + sub_rect.min.x + episode_pos_in_graph as f32, sub_rect.min.y), pos2(2.0 + sub_rect.min.x + episode_pos_in_graph as f32, sub_rect.max.y));
          let position_on_graph = Shape::rect_filled(rect_should_be_at, 0.0, Color32::from_rgba_premultiplied(0, 255, 0, 255));
          ui.painter().add(position_on_graph);
        }
        // TODO: fix input
        ui.input(|inpt|
        {

          if inpt.pointer.any_click() {
            if let Some(click_pos) = inpt.pointer.interact_pos() {
              let episode_index = get_episode_index(episode_amount, &graph_rect, &click_pos, 0);
              if let Some(new_index) = episode_index {
                let to_replay = {
                  let run_data = state.loaded_run.as_mut().unwrap();
                  if run_data.episodes.len() > new_index {
                    Some((run_data.episodes.index(new_index).clone(), run_data.env.clone(), run_data.env_config.clone()))
                  } else { None }
                };
                if let Some(replay_data) = to_replay {
                  nextstate.set(GameState::ChooseTraining);
                  let data = replay_data.0;
                  let mut env = LineRider3DEnv::default();
                  env.load_config(&replay_data.2);
                  env.use_seed(data.seed);
                  env.reset();
                  for action in &data.log {
                    env.step(action);
                  }
                  track_to_add.0 = env.lines.clone();
                  track_to_add.1 = is_freepoint_actionspace(env.sim.config.action_type);
                  track_to_add.2 = env.sim.config.goal_pos;
                  
                }
                // if to_replay.is_some() { // TODO:  check if still needed
                //   let mut renderer_state = try_write!(state.displayed_episode_state);
                //   renderer_state.to_replay = to_replay;
                //   renderer_state.current_episode.store(new_index, Ordering::Relaxed);
                //   renderer_state.current_tick.store(0, Ordering::Relaxed);
                // }
              }
            }
          }
        });
      } else {
      }
    }
  }
}
