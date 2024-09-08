pub mod plot;
pub mod single;
pub mod run_list;
pub mod run_detail;
use crate::env::{LineRider3DEnv, tracks::make_freeroam_lines};
use crate::simulator::*;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiStartupSet, egui, EguiSet, EguiPlugin};
use rusty_gym::get_env_variable;

use run_list::EguiRunList;

pub fn replay_viewer() {
  let sim = LineRiderSim::default_with_ui();
  let mut env = LineRider3DEnv::new(sim, None);
  
  env.sim.app.add_plugins(ReplayUiPlugin);
  // prepare_acc_jump(&mut env);
  make_freeroam_lines(&mut env);
  // env.sim.set_physics_delta(80, 1);
  let mut track_to_add: Mut<TrackToAdd> = env.sim.app.world.resource_mut();
  track_to_add.0 = env.lines.clone();
  track_to_add.1 = true;
  env.sim.app.run();
}


#[derive(Component)]
pub struct ResetButton;
#[derive(Component)]
pub struct PauseButton;


pub fn setup_menu_camera(mut commands: Commands) {
  commands.spawn_empty().insert(Camera3dBundle {
    transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ..default()
  }).insert(Camera).insert(ChooseEpisode);
}

struct ReplayUiPlugin;

impl Plugin for ReplayUiPlugin {
    fn build(&self, app: &mut App) {
        let file_name = get_env_variable("TLF_REPLAY_PATH")
        .unwrap_or("/workspaces/linerider/trl-experiments/LineRider3D-Env-v0/heu".to_owned());
        app.add_plugins(EguiPlugin);
        app
        .insert_resource(EguiRunList::new(file_name))
        .add_systems(OnEnter(GameState::ChooseTraining), setup_menu_camera)
        .add_systems(Update, setup_egui_ui.after(EguiStartupSet::InitContexts).after(EguiSet::BeginFrame))
        ;
    }
}

pub fn setup_egui_ui(mut egui_context: EguiContexts, mut run_list: ResMut<EguiRunList>, track_to_add: ResMut<TrackToAdd>, nextstate: ResMut<NextState<GameState>>, mut config: ResMut<LineRiderConfig>) {
  egui::Window::new("Replay List").vscroll(true)
    .show(egui_context.ctx_mut(), |ui| {
        run_list.ui(ui, nextstate, track_to_add, config);
    });
}