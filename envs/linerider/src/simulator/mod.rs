pub mod config;

pub use config::*;
pub mod system;
pub use system::*;

use bevy::{
  app::{App, First, FixedUpdate, Last, PostStartup, PostUpdate, PreStartup, PreUpdate, RunFixedUpdateLoop, ScheduleRunnerPlugin, Startup, StateTransition}, asset::AssetPlugin, ecs::schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter, OnExit, Schedule, ScheduleLabel, States}, input::keyboard::KeyCode, log::LogPlugin, math::Vec3, pbr::PbrPlugin, prelude::{Entity, FrameCountPlugin, ImagePlugin, Mut, PluginGroup, SystemSet, TaskPoolPlugin, Transform, TypeRegistrationPlugin, Update}, render::{settings::WgpuSettings, RenderPlugin}, scene::ScenePlugin, transform::TransformPlugin, utils::default, window::WindowPlugin, DefaultPlugins
};
use bevy_rapier3d::{
  geometry::Collider, prelude::{RapierPhysicsPlugin, NoUserData, 
    Velocity,  RapierConfiguration, TimestepMode}, render::RapierDebugRenderPlugin
};
use crate::{
  faketimer::{FakeTimePlugin, FakeTimeStepSize},
  util::{consts::REWARD_REACH_CHECKPOINT, range::Range3D, track::*, vel_to_f32}
};
use bevy_flycam::*;


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    ChooseTraining,
    ChooseEpisode,
    #[default]
    InSimulation
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
enum RunLabels {
  SetupDriver,
  SetupTrack,
  _Runtime
}

impl Default for LineRiderSim {
  fn default() -> LineRiderSim {
    LineRiderSim::new(false)
  }
}

pub struct LineRiderSim {
  pub app: App,
  pub config: LineRiderConfig,
  pub driver_id: Entity,
  pub with_ui: bool,
  pub origin: Option<Vec3>,
  pub build_range: Range3D<f32>,
  pub simulation_range: Range3D<f32>
}


pub fn make_app_singlethreaded(app: &mut App) {
  let schedule_setter = |schedule: &mut Schedule| {
    schedule.set_executor_kind(bevy::ecs::schedule::ExecutorKind::SingleThreaded);
  };
  app.edit_schedule(PreStartup, schedule_setter);
  app.edit_schedule(Startup, schedule_setter);
  app.edit_schedule(PostStartup, schedule_setter);
  app.edit_schedule(First, schedule_setter);
  app.edit_schedule(StateTransition, schedule_setter);
  app.edit_schedule(RunFixedUpdateLoop, schedule_setter);
  app.edit_schedule(FixedUpdate, schedule_setter);
  app.edit_schedule(PostUpdate, schedule_setter);
  app.edit_schedule(Update, schedule_setter);
  app.edit_schedule(Last, schedule_setter);
}

impl LineRiderSim  {
  pub fn default_with_ui() -> LineRiderSim {
    LineRiderSim::new(true)
  }
  pub fn prepare_app(config: LineRiderConfig, with_ui: bool) -> App {
    let mut app = App::new();
    app.insert_resource(config);
    app.insert_resource(TrackToAdd(Vec::new(), false, Vec3::ZERO));
    app.insert_resource(UseDebugCamera(false));
    app.insert_resource(ShowAABB(false));
    app.insert_resource(GoalReached(false));
    app.insert_resource(CheckpointReached(false));
    app.insert_resource(RiderTouchingTrackTimer(false, 0, 0));
    app.insert_resource(CurrentlyActiveBooster(Vec3::ZERO));
    app.insert_resource(DriverEntityRef(Entity::from_raw(0)));
    app.insert_resource(UseImageTexture(false));
    app.add_state::<GameState>();
    if with_ui {
      let plugs = DefaultPlugins.build().disable::<LogPlugin>();
      app.add_plugins(plugs);
      app.add_plugins(RapierDebugRenderPlugin::default());
      app.add_systems(Update, bevy::window::close_on_esc);
      app.add_systems(OnEnter(GameState::InSimulation), setup_light);
      app.add_systems(OnEnter(GameState::InSimulation), (setup_camera).in_set(RunLabels::SetupDriver));
      app.add_systems(Update, camera_follows_driver.run_if(in_state(GameState::InSimulation)));
      app.add_systems(Update, skybox_cubify.run_if(in_state(GameState::InSimulation)));
    } else {
      app.add_plugins(TaskPoolPlugin::default());
      app.add_plugins(TypeRegistrationPlugin);
      app.add_plugins(FrameCountPlugin);
      app.add_plugins(FakeTimePlugin);
      app.add_plugins(ScheduleRunnerPlugin::run_once());
      app.add_plugins(TransformPlugin);
      app.add_plugins(AssetPlugin::default());
      app.add_plugins(WindowPlugin::default());
      app.add_plugins(ScenePlugin);
      app.add_plugins(RenderPlugin {
        render_creation: WgpuSettings {
            backends: None,
            ..default()
        }
        .into(),
      });
      app.add_plugins(ImagePlugin::default());
      app.add_plugins(PbrPlugin::default());
      make_app_singlethreaded(&mut app); // Should improve performance for simulation
    }
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_systems(OnEnter(GameState::InSimulation), setup_rider_exclusive.in_set(RunLabels::SetupDriver).before(RunLabels::SetupTrack));
    app.add_systems(OnEnter(GameState::InSimulation), setup_goal_position_exclusive.in_set(RunLabels::SetupDriver).before(RunLabels::SetupTrack));
    app.add_systems(OnEnter(GameState::InSimulation), setup_track.after(RunLabels::SetupDriver).in_set(RunLabels::SetupTrack));
    app.add_systems(OnEnter(GameState::InSimulation), setup_goal_mesh.after(RunLabels::SetupTrack));
    app.add_systems(OnEnter(GameState::InSimulation), setup_checkpoint_mesh.after(RunLabels::SetupTrack));
    app.add_systems(OnExit(GameState::InSimulation), despawn_with::<InSimulation>);
    app.add_systems(OnExit(GameState::InSimulation), despawn_with::<Booster>);
    app.add_systems(OnExit(GameState::InSimulation), despawn_with::<Collider>);
    app.add_systems(OnExit(GameState::InSimulation), despawn_with::<Transform>);
    
    
    app.add_systems(OnEnter(GameState::ChooseTraining), switch_back_to_simuation);
    app.add_systems(OnEnter(GameState::ChooseEpisode), switch_back_to_simuation2);
    app.add_systems(OnExit(GameState::ChooseTraining), despawn_with::<ChooseEpisode>);
    app.add_systems(Update, boost_driver_on_booster_collision.run_if(in_state(GameState::InSimulation)));
    app.add_systems(Update, measure_rider_touching_track_time.run_if(in_state(GameState::InSimulation)));
    app.add_systems(Update, check_goal_reached.run_if(in_state(GameState::InSimulation)));
    app.add_systems(Update, check_checkpoint_reached.run_if(in_state(GameState::InSimulation)));
    app.add_systems(Update, check_goal_no_rapier.run_if(in_state(GameState::InSimulation)));
    app.add_systems(Update, check_checkpoint_reached_no_rapier.run_if(in_state(GameState::InSimulation)));  

    app
  }
  pub fn new(with_ui: bool) -> LineRiderSim {
    let mut config = LineRiderConfig::default();
    config.goal_pos = make_goal_pos(&config.goal_position);
    let mut sim = LineRiderSim {
      build_range: make_build_range(&config, 0.5, &None),
      simulation_range: make_build_range(&config, 0.9, &None),
      app: LineRiderSim::prepare_app(config.clone(), with_ui),
      config,
      driver_id: Entity::from_raw(0),
      with_ui,
      origin: None
    };
    if !with_ui {
      sim.set_physics_delta(sim.config.physics_delta, sim.config.physics_substeps);
      // let fake_time_step = FakeTimeStepSize::default();
      // sim.set_fake_delta(fake_time_step.0);
    }
    sim
  }
  pub fn set_max_width(&mut self, max_width: f32) {
    self.config.max_width = max_width;
    self.build_range = make_build_range(&self.config, 0.5, &self.origin);
    self.simulation_range = make_build_range(&self.config, 0.9, &self.origin);
  }
  pub fn set_fake_delta(&mut self, new_delta: u64) {
    {
      let mut fake_time_step_size: Mut<FakeTimeStepSize> = self.app.world.resource_mut();
      let old_delta = fake_time_step_size.0 as f32;
      let mut modifier = if new_delta as f32 > old_delta {1.0} else {0.0};
      modifier += new_delta as f32 / old_delta;
      fake_time_step_size.0 = new_delta;
      self.config.booster_strength = self.config.booster_strength * modifier;
    }
  }
  pub fn set_physics_delta(&mut self, new_delta: u64, substeps: usize) {
    {
      let mut rapier_config: Mut<RapierConfiguration> = self.app.world.resource_mut();
      rapier_config.timestep_mode = TimestepMode::Fixed { dt: new_delta as f32 / 1000.0, substeps };
    }
  }
  pub fn add_debug_cam(&mut self) {
    self.app.add_plugins(NoCameraPlayerPlugin);
    self.app.insert_resource(MovementSettings {
      sensitivity: 0.00015, // default: 0.00012
      speed: 12.0, // default: 12.0
    });
    self.app.insert_resource(KeyBindings {
      ..Default::default()
    });
    let mut use_debug_cam: Mut<UseDebugCamera> = self.app.world.resource_mut();
    use_debug_cam.0 = true;
  }
  pub fn reset_state(&mut self) {
    if self.with_ui {
      self.app.world.clear_trackers();
      let mut nextstate: Mut<NextState<GameState>> = self.app.world.resource_mut();
      nextstate.set(GameState::ChooseTraining);
    } else {
      self.app = LineRiderSim::prepare_app(self.config.clone(), self.with_ui);
    }
  }
  pub fn simulation_step(&mut self) {
    self.app.update();
  }
  pub fn set_goal_position(&mut self, goal_pos: Vec3) {
    let mut track_to_add: Mut<TrackToAdd> = self.app.world.resource_mut();
    track_to_add.2 = goal_pos;
    self.config.goal_position = make_goal_range(&goal_pos, &self.config);
    self.config.goal_pos = goal_pos;
  }
  pub fn get_driver_transform(&self) -> &Transform {
    let driver_entity: &DriverEntityRef = self.app.world.resource();
    let driver_ref = self.app.world.entity(driver_entity.0);
    driver_ref.get::<Transform>().expect("driver entity has pos")
  }
  pub fn get_driver_velocity(&self) -> &Velocity {
    let driver_entity: &DriverEntityRef = self.app.world.resource();
    let driver_ref = self.app.world.entity(driver_entity.0);
    driver_ref.get::<Velocity>().expect("driver entity has pos")
  }
  /**
  Reward Design: 
  Goal (Optional):
  1. make track end in specific area
  2. make track end in specific area and try to achieve specific or no speed so stop the agent there
  3. no goal just optimizie physics measures
  Negative reward if Goal never reached, otherwise combination of:

  1. Velocity at and => The faster the rider is while reaching the goal the better
  2. Overall Velocity => Summarize all experienced velocities to encourage fast rides
  3. Overall Rotation => Enocurage loopings, flips or spinnin the entity around by making rotational changes a positive reward
  4. total gain in height to encourage jumping

  (Optional)
  - different track meshes / colliders, e.g. tube to drive through
  - amount of used track => use more / less track to see how minimal / full the map can be
  - close calls => encourage going through small holes with little space top and bottom for nice visuals
  - amount of track touches => encourage more flying with momentum by discouraging the collision with track and rider or the other way around to discourage excessive flying
  **/
  pub fn simulate_till_end(&mut self, max_steps: usize) -> LineRiderSimulationResult {
    {
      let mut app_config: Mut<LineRiderConfig> = self.app.world.resource_mut();
      app_config.copy_from(&self.config);
    }
    let mut steps_taken = 0;
    let mut goal_reached = false;
    let mut checkpoint_reached = false;
    let mut previous_rotation = 0.0;
    let mut overall_velocity = 0.0;
    let mut overall_rotation = 0.0;
    let mut overall_height_gain = 0.0;
    let mut previous_height = 0.5;
    let mut closest_to_goal = std::f32::MAX;
    let mut starting_goal_distance = 0.0;
    let mut starting_pos = Vec3::ZERO;
    let mut steps_without_movement = 0;
    let mut last_movement_comparison_pos = Vec3::ZERO;
    let mut ended_because_of_no_movement = false;
    // TODO: values of overall vel and rotation are dependent on delta. hhigher fps = higher numbers, lower fps = lower numbers
    'SIM_END: for i in 0..max_steps {
      self.simulation_step();
      if i == 0 {
        let driver_pos = self.get_driver_transform().translation;
        starting_pos = driver_pos;
        starting_goal_distance = driver_pos.distance(self.config.goal_pos);
        last_movement_comparison_pos = driver_pos;
      }
      steps_taken = i;
      overall_velocity += vel_to_f32(self.get_driver_velocity());
      
      let position = self.get_driver_transform();
      let rotation_angle =  position.rotation.to_axis_angle().1;
      let rotation = (rotation_angle - previous_rotation).abs();
      previous_rotation = rotation_angle;
      overall_rotation += rotation;
      let height_gain = position.translation.y - previous_height;
      if height_gain > 0.0 {
        overall_height_gain += height_gain;
      }
      previous_height = position.translation.y;
      
      
      if !checkpoint_reached {
        let app_checkpoint_reached: &CheckpointReached = self.app.world.resource();
        if app_checkpoint_reached.0 {checkpoint_reached = true;}
      }

      let app_reached_goal: &GoalReached = self.app.world.resource();
      if app_reached_goal.0 {
        goal_reached = true;
        closest_to_goal = 0.0;
        starting_goal_distance = 0.0;
        break 'SIM_END;
      } else {
        if !self.simulation_range.vec3_in_range(&position.translation) {
          ended_because_of_no_movement = true;
          break 'SIM_END;
        }
        let get_distance_to =
        if !checkpoint_reached && self.config.reward_type.contains(&REWARD_REACH_CHECKPOINT) {
          self.config.checkpoint_pos
        } else {
          self.config.goal_pos
        };
        let distance = position.translation.distance(get_distance_to);
        starting_goal_distance = starting_pos.distance(get_distance_to);
        if distance < closest_to_goal {
          closest_to_goal = distance;
        }
      }
      if last_movement_comparison_pos.distance(self.get_driver_transform().translation) > self.config.premature_end_min_distance {
        last_movement_comparison_pos = self.get_driver_transform().translation;
        steps_without_movement = 0;
      } else {
        steps_without_movement += 1;
        // if steps_without_movement >= self.config.premature_end_after_steps_without_movement {
        //   ended_because_of_no_movement = true;
        //   break 'SIM_END;
        // }
      }
    }
    let velocity_at_end = vel_to_f32(self.get_driver_velocity()); // velocity[0].abs() + velocity[1].abs();

    let touch_timer: &RiderTouchingTrackTimer = self.app.world.resource();
    let time_rider_touched_track = touch_timer.2;
    let time_rider_airborne = touch_timer.1 - touch_timer.2;
    let total_time = touch_timer.1;

    LineRiderSimulationResult {
      steps_taken, goal_reached, velocity_at_end, overall_velocity, overall_rotation,
      overall_height_gain, checkpoint_reached,
      closest_to_goal: starting_goal_distance - closest_to_goal,
      ended_because_of_no_movement, time_rider_touched_track, time_rider_airborne, total_time
    }
  }

}
