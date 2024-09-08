use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}, ecs::query::QueryComponentError, core_pipeline::Skybox, render::render_resource::{TextureViewDescriptor, TextureViewDimension}, asset::LoadState};
use bevy_flycam::FlyCam;
use bevy_rapier3d::prelude::{RigidBody, Collider, GravityScale, ComputedColliderShape,
    Sleeping, Ccd, ColliderMassProperties, Velocity, Restitution, Friction, ActiveEvents, CollisionEvent, Sensor, RapierContext};
use nalgebra::Point3;
use super::GameState;
use crate::util::{consts::*, track::*};
use super::LineRiderConfig;
use std::{num, ops::Sub};
#[derive(Component)]
pub struct StaticMesh;

#[derive(Component)]
pub struct Booster(pub Vec3, pub bool);

#[derive(Component)]
pub struct MovingMesh;

#[derive(Component)]
pub struct GoalMesh;

#[derive(Component)]
pub struct CheckpointMesh;

#[derive(Component)]
pub struct Camera;

#[derive(Component)]
pub struct Light;

pub type TrackPoint = (Vec3, u8);

#[derive(Component)]
pub struct ChooseTraining;
#[derive(Component)]
pub struct ChooseEpisode;
#[derive(Component)]
pub struct InSimulation;

#[derive(Resource)]
pub struct TrackToAdd(pub Vec<TrackPoint>, pub bool, pub Vec3);

#[derive(Resource)]
pub struct UseImageTexture(pub bool);

#[derive(Resource)]
pub struct DriverEntityRef(pub Entity);

#[derive(Resource)]
pub struct UseDebugCamera(pub bool);

#[derive(Resource)]
pub struct ShowAABB(pub bool);

#[derive(Resource)]
pub struct GoalReached(pub bool);

#[derive(Resource)]
pub struct CheckpointReached(pub bool);

/**
 * 1 = Currently touching track
 * 2 = total time driving
 * 3 = time touching ground
 */
#[derive(Resource)]
pub struct RiderTouchingTrackTimer(pub bool, pub u64, pub u64);

#[derive(Resource)]
pub struct CurrentlyActiveBooster(pub Vec3);

#[derive(Resource)]
pub struct Cubemap {
    is_loaded: bool,
    _index: usize,
    image_handle: Handle<Image>,
}

pub fn setup_camera(mut commands: Commands, track_to_add: Res<TrackToAdd>, asset_server: Res<AssetServer>) {
  let initial_pos: Vec3 = {
    if track_to_add.0.len() > 0 {
      track_to_add.0[0].0.clone()
    } else {Vec3::new(0.0, 0.0, 0.0)}
  };
  let second_pos: Vec3 = {
    if track_to_add.0.len() > 1 {
      track_to_add.0[1].0.clone()
    } else {Vec3::new(0.0, 0.0, 0.0)}
  };
  let direction = (second_pos - initial_pos) * -2.0;
  let camera_pos = initial_pos + direction;
  let skybox_handle = asset_server.load("cubemap.png");
  commands.spawn_empty().insert(Camera3dBundle {
    transform: Transform::from_xyz(camera_pos.x, camera_pos.y + 1.5, camera_pos.z).looking_at(initial_pos, Vec3::Y),
    ..default()
  }).insert(Camera).insert(FlyCam).insert(InSimulation)
  .insert(Skybox(skybox_handle.clone()));
  commands.insert_resource(Cubemap {
    is_loaded: false,
    _index: 0,
    image_handle: skybox_handle,
  });
}

pub fn skybox_cubify(
  asset_server: Res<AssetServer>,
  mut images: ResMut<Assets<Image>>,
  mut cubemap: ResMut<Cubemap>,
  mut skyboxes: Query<&mut Skybox>,
  use_debug_camera: Res<UseDebugCamera>
) {
  if !use_debug_camera.0 && !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle) == LoadState::Loaded {
      let image = images.get_mut(&cubemap.image_handle).unwrap();
      // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
      // so they appear as one texture. The following code reconfigures the texture as necessary.
      if image.texture_descriptor.array_layer_count() == 1 {
          image.reinterpret_stacked_2d_as_array(image.height() / image.width());
          image.texture_view_descriptor = Some(TextureViewDescriptor {
              dimension: Some(TextureViewDimension::Cube),
              ..default()
          });
      }

      for mut skybox in &mut skyboxes {
          skybox.0 = cubemap.image_handle.clone();
      }

      cubemap.is_loaded = true;
  }
}

pub fn setup_light(mut commands: Commands) {
  commands.spawn_empty().insert(DirectionalLightBundle {
    directional_light: DirectionalLight {
        shadows_enabled: false,
        ..default()
    },
    transform: Transform {
        translation: Vec3::new(0.0, 2.0, 0.0),
        rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.),
        ..default()
    },
    ..default()
  }).insert(InSimulation);
}

pub fn check_goal_reached(rapier_context: Res<RapierContext>, mut goal_reached: ResMut<GoalReached>, mut set: ParamSet<(Query<(Entity, With<MovingMesh>)>, Query<(Entity, With<GoalMesh>)>)>) {
  if set.p1().get_single().is_ok() && set.p0().get_single().is_ok() {
    let e1 = {
      let entity1 = set.p0();
      entity1.single().0
    };
    let e2 = {
      let entity2 = set.p1();
      entity2.single().0
    };
    if rapier_context.intersection_pair(e1, e2) == Some(true) {
      goal_reached.0 = true;
    }
  }
}
pub fn check_checkpoint_reached(rapier_context: Res<RapierContext>, mut checkpoint_reached: ResMut<CheckpointReached>, mut set: ParamSet<(Query<(Entity, With<MovingMesh>)>, Query<(Entity, With<CheckpointMesh>)>)>) {
  if set.p1().get_single().is_ok() && set.p0().get_single().is_ok() {
    let e1 = {
      let entity1 = set.p0();
      entity1.single().0
    };
    let e2 = {
      let entity2 = set.p1();
      entity2.single().0
    };
    if rapier_context.intersection_pair(e1, e2) == Some(true) {
      checkpoint_reached.0 = true;
    }
  }
}

pub fn check_checkpoint_reached_no_rapier(mut checkpoint_reached: ResMut<CheckpointReached>, config_res: Res<LineRiderConfig>, rider_query: Query<&mut Transform, With<MovingMesh>>) {
  let rider_transform = rider_query.single();
  if config_res.checkpoint_range.vec3_in_range(&rider_transform.translation) {
    checkpoint_reached.0 = true;
  }
}
 
pub fn check_goal_no_rapier(mut goal_reached: ResMut<GoalReached>, config_res: Res<LineRiderConfig>, rider_query: Query<&mut Transform, With<MovingMesh>>) {
  let rider_transform = rider_query.single();
  if config_res.goal_position.vec3_in_range(&rider_transform.translation) {
    goal_reached.0 = true;
  }
}

pub fn setup_goal_mesh(config_res: Res<LineRiderConfig>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
  let config = config_res;
  let material_handle = materials.add(COLOR_GOAL.into());
  let half_goal_size = config.goal_size / 2.0;
  let shape = config.goal_position.to_box();
  let mesh = Mesh::from(shape);
  let mut entity = commands.spawn_empty();
  entity.insert(RigidBody::Fixed);
  entity.insert(Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).expect("Can convert to collider"));
  let mesh_handle = meshes.add(mesh);
  entity.insert(Sensor);
  entity.insert(GoalMesh);
  entity.insert(InSimulation);
  entity.insert(PbrBundle {
    mesh: mesh_handle.into(),
    material: material_handle,
    transform: Transform::from_translation(Vec3::ZERO),
    ..default()
  });
}

pub fn setup_checkpoint_mesh(config_res: Res<LineRiderConfig>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
  let config = config_res;
  let material_handle = materials.add(COLOR_CHECKPOINT.into());
  let shape = config.checkpoint_range.to_box();
  let mesh = Mesh::from(shape);
  let mut entity = commands.spawn_empty();
  entity.insert(RigidBody::Fixed);
  entity.insert(Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).expect("Can convert to collider"));
  let mesh_handle = meshes.add(mesh);
  entity.insert(Sensor);
  entity.insert(CheckpointMesh);
  entity.insert(InSimulation);
  entity.insert(PbrBundle {
    mesh: mesh_handle.into(),
    material: material_handle,
    transform: Transform::from_translation(Vec3::ZERO),
    ..default()
  });
}

pub fn setup_rider_exclusive(world: &mut World) {
  let config: LineRiderConfig = {let config: &LineRiderConfig = world.resource(); config.clone()};
  let use_image_texture = {
    let res: &UseImageTexture = world.resource();
    res.0
  };
  
  let material_handle: Handle<StandardMaterial> = if use_image_texture {
    let asset_server: &AssetServer = world.resource();
    asset_server.load("dirty_football_2k.gltf#Material0")
  } else {
    let mut materials: Mut<Assets<StandardMaterial>> = world.resource_mut();
    let material_handle = materials.add(COLOR_RIDER.into());
    material_handle
  };
  let shape = match config.rider_shape {
    _ => {
      //shape::Box::new(config.rider_size[0], config.rider_size[1], config.rider_size[2])
      let mut capsule = shape::Capsule::default();
      capsule.radius = config.rider_size[1];
      capsule.depth = 0.001;
      capsule
    }
  };
  let mesh_handle = {
    let mut meshes: Mut<Assets<Mesh>> = world.resource_mut();
    meshes.add(Mesh::from(shape))
  };
  let start_pos_mesh = {
    let mut meshes: Mut<Assets<Mesh>> = world.resource_mut();
    meshes.add(Mesh::from(shape::Box::new(0.2, 0.2, 0.2)))
  };
  let start_pos_material = {
    let mut materials: Mut<Assets<StandardMaterial>> = world.resource_mut();
    let material_handle = materials.add(COLOR_SPAWN.into());
    material_handle
  };
  let mut initial_pos: Vec3 = {
    let track_to_add: &TrackToAdd = world.resource();
    if track_to_add.0.len() > 0 {
      track_to_add.0[0].0.clone()
    } else {Vec3::new(0.0, 0.0, 0.0)}
  };
  // let second_pos: Vec3 = {
  //   let track_to_add: &TrackToAdd = world.resource();
  //   if track_to_add.0.len() > 0 {
  //     track_to_add.0[1].0.clone()
  //   } else {initial_pos}
  // };
  // let mut direction = second_pos.sub(initial_pos).normalize();
  // direction *= 0.1;
  initial_pos.y += 0.4;
  // initial_pos.x += 0.1;
  {
    let mut rider_start_pos = world.spawn_empty();
    rider_start_pos.insert(InSimulation);
    rider_start_pos.insert(PbrBundle {
      mesh: start_pos_mesh.into(),
      material: start_pos_material.clone(),
      transform: Transform::from_translation(initial_pos),
      ..default()
    });
  }
  let entity_id = {
    let mut entity = world.spawn_empty();
    entity.insert(RigidBody::Dynamic);
    match config.rider_shape {
      1 => {
        entity.insert(Collider::ball(config.rider_size[1]));
      },
      _ => {
        entity.insert(Collider::cuboid(config.rider_size[0], config.rider_size[1], config.rider_size[2]));
      }
    }
    entity.insert(GravityScale(1.0));
    entity.insert(ColliderMassProperties::Mass(config.rider_density));
    entity.insert(Sleeping::disabled());
    entity.insert(Restitution::coefficient(0.5));
    entity.insert(Friction::coefficient(0.0));
    // entity.insert(ExternalForce {
    //   force: Vec3::new(3.0, 0.0, 0.0),
    //   torque: Vec3::new(1.0, 1.0, 1.0),
    // });
    entity.insert(Velocity {
      linvel: Vec3::new(0.0, 0.0, 0.0),
      angvel: Vec3::ZERO
    });
    // entity.insert(TransformBundle::from(Transform::from_xyz(2.0, 3.0, 0.0)));
    entity.insert(Ccd::enabled());
    entity.insert(MovingMesh);
    entity.insert(InSimulation);
    entity.insert(ActiveEvents::COLLISION_EVENTS);
    entity.insert(PbrBundle {
      mesh: mesh_handle.into(),
      material: material_handle.clone(),
      transform: Transform::from_translation(initial_pos),
      ..default()
    });
    entity.id()
  };
  let mut driver_ref: Mut<DriverEntityRef> = world.resource_mut();
  driver_ref.0 = entity_id;
}

pub fn line_plane_intersection(line_direction_v: Vec3, line_point_p: Vec3, plane_normal: Vec3, plane_d: Vec3) -> Vec3 {
  let dot1 = plane_normal.dot(line_direction_v);
  let dot2 = plane_normal.dot(line_point_p);
  let t = -(dot2 - plane_d) / dot1;
  return line_point_p + (t * line_direction_v);
}


pub fn setup_track(mut commands: Commands, track_to_add: Res<TrackToAdd>, use_image_texture: Res<UseImageTexture>,
config: Res<LineRiderConfig>, mut meshes: ResMut<Assets<Mesh>>,
mut materials: ResMut<Assets<StandardMaterial>>, asset_server: Res<AssetServer>,
mut driver_query: Query<&mut Velocity, With<MovingMesh>>, show_aabb: Res<ShowAABB>) {
  let points = &track_to_add.0;
  let material_handle = if use_image_texture.0 {
    let texture_handle = asset_server.load("textures/Grass003_4K_Color.jpg");
    let normal_handle = asset_server.load("textures/Grass003_4K_NormalGL.jpg");
    materials.add(StandardMaterial {
      // base_color: Color::GREEN.clone(),
      base_color_texture: Some(texture_handle.clone()),
      unlit: false,
      normal_map_texture: Some(normal_handle.clone()),
      fog_enabled: false,
      double_sided: true,
      alpha_mode: AlphaMode::Blend,
      perceptual_roughness: 1.0,
      cull_mode: None,
      ..default()
    })
  } else {
    let material_handle = materials.add(COLOR_TRACK.into());
    material_handle
  };
  let material_handle_boost = {
    let material_handle = materials.add(COLOR_TRACK_BOOST.into());
    material_handle
  };
  let material_handle_decelerate = {
    let material_handle = materials.add(COLOR_TRACK_BRAKE.into());
    material_handle
  };
  
  if config.use_cylinder_track {
    if points.len() > 1 {
      let prev_point = points[0].0;
      let current_point = points[1].0;
      let diff1 = current_point - prev_point;
      // Initial Force for movement
      let mut driver_velocity = driver_query.get_single_mut().unwrap();
      driver_velocity.linvel = diff1 * config.starting_force_multiplier;
      
    }
    let new_points = if config.smooth_free_points {
      catmull_rom(points, points.len() * config.bezier_resolution)
    } else {points.to_vec()};
    let np: Vec<Vec3> = new_points.iter().map(|a| {a.0.clone()}).collect();
    let num_segments = 16;
    let radius = 10.1;
    let mut end_vertices: Option<Vec<Vec3>> = None;
      // Iterate over each point in the path
      // let current_point = new_points[i].0;
      // let next_point = if i < new_points.len() - 1 {
      //   new_points[i + 1].0
      // } else {
      //   new_points[i].0
      // };
      // let (vertices_1, indices, prev_end) = generate_pipe_vertices(
      //   current_point, next_point, radius, num_segments, None
      // );
      let (vertices_1, indices) = devin(
        &np, radius, num_segments
      );
      // end_vertices = Some(prev_end);
      let vertices_to_use = vertices_1;
      let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
      let collected: Vec<[f32; 3]> = vertices_to_use.iter().map(|p| [p.x, p.y, p.z]).collect();
      let normals: Vec<[f32; 3]> = vertices_to_use.iter().map(|_p| [1.0, 1.0, 1.0]).collect();
      let uvs: Vec<[f32; 2]> = vertices_to_use.iter().map(|_p| [0.0, 0.0]).collect();
      mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, collected.clone());
      mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
      mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
      println!("INDICES ARE {:?} vertices are {:?}", indices, vertices_to_use);
      mesh.set_indices(Some(Indices::U32(indices.to_vec())));
      // let mut collider_indices: Vec<[u32; 3]> = Vec::with_capacity(indices_to_use.len()/2);
      // let mut i = 0;
      // while i < indices.len() {
      //   collider_indices.push([indices[i], indices[i+1], indices[i+2]]);
      //   i += 3
      // }
      let color_to_use = COLOR_TRACK.with_a(0.5);
      let material_handle_purple = materials.add(color_to_use.into());
      let mut entity = commands.spawn_empty();
      entity.insert(RigidBody::Fixed)
      .insert(Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).expect("collidermakeable"))
      // .insert(Sleeping::disabled())
      .insert(InSimulation);
  
      let mesh_handle = meshes.add(mesh.into());
      entity.insert(PbrBundle {
        mesh: mesh_handle.into(),
        material: material_handle_purple.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
      });

  } else {
    // let material_handle = asset_server.load("herringbone_parquet_1k.gltf#Material0");
    if track_to_add.1 {
      let mut i = 1;
      if points.len() > 1 {
        let prev_point = points[0].0;
        let current_point = points[1].0;
        let diff1 = current_point - prev_point;
        // Initial Force for movement
        if i == 1 {
          let mut driver_velocity = driver_query.get_single_mut().unwrap();
          driver_velocity.linvel = diff1 * config.starting_force_multiplier;
        }
      }
      let new_points = if config.smooth_free_points {
        catmull_rom(points, points.len() * config.bezier_resolution)
      } else {points.to_vec()};
      let mut prev_left = Vec3::ZERO;
      let mut prev_right = Vec3::ZERO;
      let mut prev_left_wall = Vec3::ZERO;
      let mut prev_right_wall = Vec3::ZERO;
      while i < new_points.len() {
        if new_points[i].1 != TP_EMPTY {
          let (all_points, diff1) = get_free_mesh_points(new_points[i-1].0, new_points[i].0, &[prev_left, prev_right, prev_left_wall, prev_right_wall], &config);
          prev_left = all_points[1][1];
          prev_left_wall = all_points[3][1];
          prev_right = all_points[2][1];
          prev_right_wall = all_points[4][1];
          let (mesh, mesh_points) = make_track_mesh(&all_points, 1);

          if show_aabb.0 {
            let mesh_aabb = mesh.compute_aabb().unwrap();
            commands.spawn_empty().insert(RigidBody::Fixed)
            .insert(Collider::cuboid( mesh_aabb.half_extents.x,  mesh_aabb.half_extents.y,  mesh_aabb.half_extents.z))
            .insert(Sleeping::disabled())
            .insert(TransformBundle::from_transform(Transform::from_translation(Vec3::new(mesh_aabb.center.x, mesh_aabb.center.y, mesh_aabb.center.z))))
            .insert(InSimulation);
          }

          let mesh_handle = meshes.add(mesh.into());
          let material_to_use = match points[i].1 {
            TP_ACCELERATE => {material_handle_boost.clone()},
            TP_DECELERATE => {material_handle_decelerate.clone()},
            _ => {material_handle.clone()}
          };
          let mut entity = commands.spawn_empty();
          entity.insert(RigidBody::Fixed)
          .insert(Collider::trimesh(mesh_points, MESH_INDICES_COLLIDER.to_vec()))
          .insert(Sleeping::disabled())
          .insert(InSimulation)
          .insert(PbrBundle {
            mesh: mesh_handle.into(),
            material: material_to_use,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
          });
          if new_points[i].1 >= TP_ACCELERATE {
            let sign_changer = if new_points[i].1 == TP_DECELERATE {-1.0} else {1.0};
            entity.insert(Booster(diff1 * sign_changer * config.booster_strength, false));
          }
        }
        i += 1;
      }
    } else {
      let mut prev_direction = DIRECTION_FORWARD;
      let mut i = 1;
      while i < points.len() {
        let prev_real_point = points[i-1].0;
        let mut current_real_point = points[i].0;
        let diff1 = current_real_point - prev_real_point;
        // Initial Force for movement
        if i == 1 {
          let mut driver_velocity = driver_query.get_single_mut().unwrap();
          driver_velocity.linvel = diff1 * config.starting_force_multiplier;
        }
        let current_direction = get_direction(prev_direction, prev_real_point, current_real_point);
        if points[i].1 != TP_EMPTY {
          let mut u = i + 1;
          if diff1.y != 0.0 { // workaround to prevent left right combination to be seen as one big left curve
            'POINT_CONNECTOR: while u < points.len() {
              let next_point = points[u].0;
              let diff2 = next_point - current_real_point;
              if diff1 == diff2 {
                current_real_point = next_point;
                i = u;
              } else {
                break 'POINT_CONNECTOR;
              }
              u += 1;
            }
          }
          // diff1 = current_real_point - prev_real_point;
          let current_action = get_action(prev_direction, prev_real_point, current_real_point);
          let all_points = get_all_points(prev_real_point, current_real_point, prev_direction, current_direction, current_action, &config);
          for u in 1..all_points[0].len() {
            let (mesh, mesh_points) = make_track_mesh(&all_points, u);

            if show_aabb.0 {
              let mesh_aabb = mesh.compute_aabb().unwrap();
              commands.spawn_empty().insert(RigidBody::Fixed)
              .insert(Collider::cuboid( mesh_aabb.half_extents.x,  mesh_aabb.half_extents.y,  mesh_aabb.half_extents.z))
              .insert(Sleeping::disabled())
              .insert(TransformBundle::from_transform(Transform::from_translation(Vec3::new(mesh_aabb.center.x, mesh_aabb.center.y, mesh_aabb.center.z))))
              .insert(InSimulation);
            }

            let mesh_handle = meshes.add(mesh.into());
            // let material_handle = materials.add(Color::PURPLE.into());
            let material_to_use = match points[i].1 {
              TP_ACCELERATE => {material_handle_boost.clone()},
              TP_DECELERATE => {material_handle_decelerate.clone()},
              _ => {material_handle.clone()}
            };
            let mut entity = commands.spawn_empty();
            entity.insert(RigidBody::Fixed)
            .insert(Collider::trimesh(mesh_points, MESH_INDICES_COLLIDER.to_vec()))
            .insert(Sleeping::disabled())
            .insert(PbrBundle {
              mesh: mesh_handle.into(),
              material: material_to_use,
              transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
              ..default()
            }).insert(InSimulation);
            if points[i].1 >= TP_ACCELERATE {
              let sign_changer = if points[i].1 == TP_DECELERATE {-1.0} else {1.0};
              entity.insert(Booster((diff1*1.1) * sign_changer * config.booster_strength, false));
            }
          }
        }
        prev_direction = current_direction;
        i += 1;
      }
    }
  }
}

pub fn setup_goal_position_exclusive(world: &mut World) {
  let goal_pos = {
    let track_to_add: &TrackToAdd = world.resource();
    track_to_add.2
  };
  let mut config: Mut<LineRiderConfig> = world.resource_mut();
  config.goal_position = make_goal_range(&goal_pos, &config);
  config.goal_pos = goal_pos;
}

pub fn light_follows_ball(mut set: ParamSet<(
  Query<&mut Transform, With<MovingMesh>>,
  Query<&mut Transform, With<Light>>
)>) {
  let rider_transform = {
    let p0 = set.p0();
    p0.single().clone()
  };
  let mut p1 = set.p1();
  let mut light_transform = p1.single_mut();
  light_transform.translation[0] = rider_transform.translation.x + 10.0;
  light_transform.translation[1] = rider_transform.translation.y + 10.0;
  light_transform.translation[2] = rider_transform.translation.z;
}

pub fn camera_follows_driver(use_debug_camera: Res<UseDebugCamera>,mut set: ParamSet<(
  Query<&mut Transform, With<MovingMesh>>,
  Query<&mut Transform, With<Camera>>
)>) {
  if !use_debug_camera.0 {
    let rider_transform = {
      let p0 = set.p0();
      p0.single().clone()
    };
    let mut p1 = set.p1();
    let mut camera_transform = p1.single_mut();
    camera_transform.translation[0] = rider_transform.translation.x - 2.0;
    camera_transform.translation[1] = rider_transform.translation.y + 4.0;
    camera_transform.translation[2] = rider_transform.translation.z + 2.0;
    *camera_transform = camera_transform.looking_at(rider_transform.translation, Vec3::Y);
  }
}

pub fn measure_rider_touching_track_time(
  time: Res<Time::<Virtual>>,
  mut timer: ResMut<RiderTouchingTrackTimer>,
  mut collision_events: EventReader<CollisionEvent>,
  driver_query: Query<&MovingMesh>
) {
  for collision_event in collision_events.read() {
      match collision_event {
        CollisionEvent::Started(e1, _e2, _s) => {
          if driver_query.get_component::<MovingMesh>(e1.clone()).is_ok() {
            timer.0 = true;
          }
        },
        CollisionEvent::Stopped(e1, _e2, _s) => {
          if driver_query.get_component::<MovingMesh>(e1.clone()).is_ok() {
            timer.0 = false;
          }
        }
      }
  }
  let delta = time.delta().as_millis() as u64;
  if timer.0 {
    timer.2 += delta;
  }
  timer.1 += delta;
}

pub fn boost_driver_on_booster_collision(
  mut collision_events: EventReader<CollisionEvent>,
  mut active_booster: ResMut<CurrentlyActiveBooster>,
  mut driver_query: Query<&mut Velocity, With<MovingMesh>>,
  is_driver_query: Query<&MovingMesh>,
  mut booster_query: Query<&mut Booster>
) {

  for collision_event in collision_events.read() {
    match collision_event {
      CollisionEvent::Started(e1, e2, _s) => {
        let booster_res: Result<Mut<Booster>, QueryComponentError> = booster_query.get_component_mut(e2.clone());
        if let Ok(booster) = booster_res {
          if is_driver_query.get_component::<MovingMesh>(e1.clone()).is_ok() {
            active_booster.0 = booster.0;
          }
        }
      },
      CollisionEvent::Stopped(e1, _e2, _s) => {
        if driver_query.get_component::<MovingMesh>(e1.clone()).is_ok() {
          active_booster.0 = Vec3::ZERO;
        }
      }
    }
  }
  if active_booster.0 != Vec3::ZERO {
    let mut driver_velocity = driver_query.get_single_mut().unwrap();
    driver_velocity.linvel += active_booster.0;
  }
}

pub fn switch_back_to_simuation(mut nextstate: ResMut<NextState<GameState>>) {
  nextstate.set(GameState::ChooseEpisode);
}
pub fn switch_back_to_simuation2(mut nextstate: ResMut<NextState<GameState>>) {
  nextstate.set(GameState::InSimulation);
}

/// Despawn all entities with a given component type
pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
  for e in q.iter() {
      commands.entity(e).despawn_recursive();
  }
}