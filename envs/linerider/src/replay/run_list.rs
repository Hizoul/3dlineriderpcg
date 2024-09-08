use bevy::{prelude::{Resource, ResMut}, ecs::schedule::NextState};
use bevy_egui::egui::{Button, Color32, Context, Frame, Stroke, Ui};
use std::sync::{Arc, RwLock};
use rusty_gym::eval::{AnalyzerIndex, AnalyzerEntry};
#[cfg(not(target_arch = "wasm32"))]
use rusty_gym::eval::create_run_index;
use crate::{try_read, try_write, simulator::{system::TrackToAdd, GameState}};
use super::{
  // run_search::EguiRunSearch,
  run_detail::{EguiRunDetail, LockedEguiRunDetailState}, LineRiderConfig};

#[derive(Resource)]
pub struct EguiRunList {
  pub state: Arc<RwLock<EguiRunListState>>,
  pub run_detail_window: EguiRunDetail
}

pub struct EguiRunListState {
  pub file_name: String,
  pub use_ipfs: bool,
  pub ipfs_base_url: String,
  pub ipfs_cid: String,
  pub file_index: Option<AnalyzerIndex>,
  pub detail_state: LockedEguiRunDetailState,
  ctx: Option<Context>
}


pub type LockedEguiRunListState = Arc<RwLock<EguiRunListState>>;

impl EguiRunListState {
  pub fn new(file_name: String, detail_state: LockedEguiRunDetailState) -> EguiRunListState {
    EguiRunListState {
      file_name, detail_state,
      use_ipfs: false,
      ipfs_base_url: "https://cloudflare-ipfs.com".to_owned(),
      ipfs_cid: "QmRVHqGX56q52pGkG5s51pBqB6n3LzjAtBqMyQThKGFZ6E".to_owned(),
      file_index: None,
      ctx: None
    }
  }
}

impl EguiRunList {
  pub fn new(file_name: String) -> EguiRunList {

    let run_detail_window = EguiRunDetail::new(file_name.clone(), false);

    let state = Arc::new(RwLock::new(EguiRunListState::new(file_name, run_detail_window.state.clone())));
    #[cfg(target_arch = "wasm32")]
    {
      wasm_bindgen_futures::spawn_local(EguiRunList::init_list(state.clone()));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
      let cloned_state = state.clone();
      // todo: use a global threadpool because this just hurts
      // std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
          let inner_runtime = tokio::runtime::Runtime::new().unwrap();
          EguiRunList::init_list(cloned_state).await;
          inner_runtime.shutdown_background();
        });
      // });
    }

    EguiRunList {
      state,
      run_detail_window
    }
  }

  pub async fn init_list(l_state: LockedEguiRunListState) {
    println!("Initializing runlist");
    let use_ipfs = {
      #[cfg(not(target_arch = "wasm32"))]
      {
        let state = try_read!(l_state);
        state.use_ipfs
      }
      #[cfg(target_arch = "wasm32")]
      {
        true
      }
    };
    println!("READING INDEX");
    let ipfs_base_url = {
      let state = try_read!(l_state);
      state.ipfs_base_url.clone()
    };
    let ipfs_cid = {
      let state = try_read!(l_state);
      state.ipfs_cid.clone()
    };
    let run_index: AnalyzerIndex = if use_ipfs {
      rusty_gym::ipfs_gateway_runs_from_cid(&ipfs_base_url, &ipfs_cid).await
    } else {
      let file_index: AnalyzerIndex = {

          #[cfg(not(target_arch = "wasm32"))] {
            let file_name = {
              let state = try_read!(l_state);
              state.file_name.clone()
            };
            create_run_index(file_name)
          }

          #[cfg(target_arch = "wasm32")]
          {
            Vec::new()
          }
        };
        file_index
    };
    let run_list = run_index;
    let mut state = try_write!(l_state);
    state.file_index = Some(run_list.clone());
    if run_list.len() > 0 {
      let run = &run_list[0];
      let mut detail_state = try_write!(state.detail_state);
      detail_state.displayed_run = if use_ipfs {

        #[cfg(target_arch = "wasm32")] {
          web_sys::console::log_1(&format!("the first run in the list has filename {}", run.load_from).into());
        }
        Some(run.load_from.clone())
      } else {Some(format!("{}.tlr", run.id))};
      detail_state.calculated_avg_rewards = false;
    }
    if let Some(ctx) = state.ctx.as_ref() {
      ctx.request_repaint();
    }
    
  }

  pub fn process_data_for_frame(&mut self, ui: &Ui) {
    let mut state = try_write!(self.state);
    if state.ctx.is_none() {
      state.ctx = Some(ui.ctx().clone());
    }
  }

  pub fn ui(&mut self, ui: &mut Ui, nextstate: ResMut<NextState<GameState>>, track_to_add: ResMut<TrackToAdd>, config: ResMut<LineRiderConfig>) {
    self.process_data_for_frame(ui);

    ui.vertical(|ui| {
      let screen_size = ui.max_rect();
      let left_part = screen_size.max.x * 0.4;
      // let right_part = screen_size.max.x * 0.6;
      // let max_height = screen_size.max.y;
      // let mut y_start = max_height * 0.02;
      // let area_size = vec2(left_part, max_height);
      // let area_size_header = vec2(screen_size.max.y, y_start);
      let state_ref = self.state.clone();
      let mut run_list: Vec<AnalyzerEntry> = Vec::new();
      let file_index = {
        let state = try_read!(state_ref);
        state.file_index.clone()
      };
      
      ui.vertical(|ui| {
        if let Some(indx) = &file_index {
          indx.iter().for_each(|run| run_list.push(run.clone()));
          {
            let state = try_read!(state_ref);
            ui.label(format!("Looking at {} runs from {}", run_list.len(), state.file_name));
          }
          // let search = EguiRunSearch::new();
          // search.ui(ui);
          let mut i = 0;
          for run in run_list {
            i += 1;
            let mut run_area = Frame::dark_canvas(&ui.style());
            run_area.fill = if i % 2 == 0 {Color32::LIGHT_GRAY} else {Color32::WHITE};
            run_area.show(ui, |ui|  {
              let frame_style = ui.style_mut();
              if i % 2 == 0 {
                frame_style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(5.0, Color32::BLACK);
              } else {
                frame_style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(5.0, Color32::BLACK);
                };
              ui.horizontal(|ui| {
                let is_training = run.run_type == 2;
                let run_type = if is_training {"🏋"} else {"💪"};
                let _alt = "Undefined".to_owned();
                ui.vertical(|ui| {
                  ui.set_width(left_part - screen_size.max.x*0.15);
                  ui.horizontal(|ui| {
                    ui.label(format!("{}", run_type));
                    ui.label(format!("{}🔄", run.episode_amount));
                    ui.label(format!("{}", run.algo));
                    if run.reused_by.is_some() {
                      ui.label(format!("♻ Continued"));
                    }
                  });
                  ui.horizontal(|ui| {
                    ui.label(format!("🌎 {}", run.env));
                    if is_training && run.reuses.is_some() {
                      ui.label(format!("♻ Transferred"));
                    }
                  });
                });
                let open_btn = Button::new("📂 View");
                // TODO: make large text
                // open_btn.text.text_style(TextStyle::Heading);
                let btn = ui.add(open_btn);
                if btn.clicked() {
                  {
                    let (_ipfs_base_url, use_ipfs) = {
                      let state = try_read!(state_ref);
                      (state.ipfs_base_url.clone(), state.use_ipfs)
                    };
                    let set_to = if use_ipfs {
                      Some(run.load_from.clone())
                    } else {
                      Some(run.load_from.clone())
                    };
                    self.run_detail_window.set_displayed_run(set_to);
                  }
                }
              });
            });
          }
        } else {
          let state = try_read!(state_ref);
          ui.label(format!("Loading Runlist. This might take a while depending on the IPFS-Gateway response time. {}", state.file_name));
        }
      });
      self.run_detail_window.ui(ui, nextstate, track_to_add, config);
    });
  }
}
