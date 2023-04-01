use bevy::{prelude::*, app::PluginGroupBuilder};
use bevy_ecs_ldtk::{LdtkLevel, LevelSelection};
use crate::{ AppState, player::Player };

const ASPECT_RATIO: f32 = 16. / 9.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
 fn build(&self, app: &mut App) {
  app
    .add_system(spawn_camera.in_schedule(OnExit(AppState::GameLoading)))
    .add_system(camera_fit_inside_current_level.in_set(OnUpdate(AppState::GameRunning)))
    ;
 }
}

fn spawn_camera(mut commands: Commands) {
  let camera = Camera2dBundle::default();
  commands.spawn(camera);
}

pub fn camera_fit_inside_current_level(
  mut camera_query: Query<
      (
          &mut bevy::render::camera::OrthographicProjection,
          &mut Transform,
      ),
      Without<Player>,
  >,
  player_query: Query<&Transform, With<Player>>,
  level_query: Query<
      (&Transform, &Handle<LdtkLevel>),
      (Without<OrthographicProjection>, Without<Player>),
  >,
  level_selection: Res<LevelSelection>,
  ldtk_levels: Res<Assets<LdtkLevel>>,
) {
  if let Ok(Transform {
      translation: player_translation,
      ..
  }) = player_query.get_single()
  {
      let player_translation = *player_translation;

      let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();

      for (level_transform, level_handle) in &level_query {
          if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
              let level = &ldtk_level.level;
              if level_selection.is_match(&0, level) {
                  let level_ratio = level.px_wid as f32 / ldtk_level.level.px_hei as f32;
                  orthographic_projection.viewport_origin = Vec2::ZERO;
                  if level_ratio > ASPECT_RATIO {
                      // level is wider than the screen
                      let height = (level.px_hei as f32 / 9.).round() * 9.;
                      let width = height * ASPECT_RATIO;
                      orthographic_projection.scaling_mode =
                          bevy::render::camera::ScalingMode::Fixed { width, height };
                      camera_transform.translation.x =
                          (player_translation.x - level_transform.translation.x - width / 2.)
                              .clamp(0., level.px_wid as f32 - width);
                      camera_transform.translation.y = 0.;
                  } else {
                      // level is taller than the screen
                      let width = (level.px_wid as f32 / 16.).round() * 16.;
                      let height = width / ASPECT_RATIO;
                      orthographic_projection.scaling_mode =
                          bevy::render::camera::ScalingMode::Fixed { width, height };
                      camera_transform.translation.y =
                          (player_translation.y - level_transform.translation.y - height / 2.)
                              .clamp(0., level.px_hei as f32 - height);
                      camera_transform.translation.x = 0.;
                  }

                  camera_transform.translation.x += level_transform.translation.x;
                  camera_transform.translation.y += level_transform.translation.y;
              }
          }
      }
  } else {
    println!("Player not spawned yet")
  }
}

// This system shows how to respond to a window being resized.
// Whenever the window is resized, the text will update with the new resolution.
// fn on_resize_system(
//   // windows: Query<&Window>,
//   level_query: Query<&LevelInfo>,
//   mut text_query: Query<&mut Text, With<ResolutionText>>,
//   mut resize_reader: EventReader<WindowResized>,
//   mut camera_query: Query<&mut Transform, With<Camera2d>>,
// ) {
//   let mut text = text_query.single_mut();
//   let mut camera_transform = camera_query.single_mut();
//   let level = level_query.single();

//   for e in resize_reader.iter() {
//       // When resolution is being changed
//       text.sections[0].value = format!("{:.1} x {:.1}", e.width, e.height);
//       // Update camera position
//       camera_transform.scale = Vec3::new(level.width / e.width,  level.height / e.height, 1.0);
//   }
// }