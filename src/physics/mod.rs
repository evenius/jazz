use bevy::{prelude::*, app::PluginGroupBuilder, utils::HashSet};
use bevy_ecs_ldtk::EntityInstance;
use bevy_rapier2d::prelude::*;

mod collision;
pub use collision::*;

use crate::AppState;
pub struct PhysicsPluginGroup;

impl PluginGroup for PhysicsPluginGroup {
  fn build(self) -> PluginGroupBuilder {
      PluginGroupBuilder::start::<Self>()
        .add(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add(PhysicsPlugin)
  }
}


pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
 fn build(&self, app: &mut App) {
  app
    .insert_resource(RapierConfiguration {
        gravity: Vec2::new(0.0, -2000.0),
        ..Default::default()
    })
    .add_system(update_on_ground.in_set(OnUpdate(AppState::GameRunning)))
    .add_system(ground_detection.in_set(OnUpdate(AppState::GameRunning)))
    .add_system(spawn_ground_sensor.in_set(OnUpdate(AppState::GameRunning)))
    .add_system(spawn_wall_collision.in_set(OnUpdate(AppState::GameRunning)))
    ;
  //   .add_system(spawn_player.in_schedule(OnEnter(AppState::GameRunning)))
  //   .add_system(load_sprites.in_schedule(OnEnter(AppState::GameLoading)))
  //   .add_system(debug_level.in_set(OnUpdate(AppState::GameRunning)));
  //   // .add_system(camera_fit_inside_current_level.in_set(OnUpdate(AppState::GameRunning)));
 }
}