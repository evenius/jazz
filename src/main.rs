use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkWorldBundle, LdtkPlugin, LevelSelection, LdtkSettings, LevelSpawnBehavior, SetClearColor, LdtkAsset, LdtkSystemSet};
use bevy_rapier2d::prelude::PhysicsSet;

mod level;
mod loading;
mod player;
mod physics;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    Menu,
    #[default]
    GameLoading,
    GameRunning,
    GamePaused
}

pub const SHOW_COLLIDER_BOXES: bool = false;

fn main () {
  App::new()
    .add_plugins(DefaultPlugins.set(
      WindowPlugin {
        primary_window: Some(Window {
            title: "I am a window!".into(),
            resolution: (1024., 640.).into(),
            ..default()
        }),
        ..default()
    }).set(ImagePlugin::default_nearest()))
    .add_state::<AppState>()
    .add_plugin(DebugStatePlugin)
    .add_plugins(level::LevelPluginGroup)
    .add_plugins(physics::PhysicsPluginGroup)
    .configure_set(LdtkSystemSet::ProcessApi.before(PhysicsSet::SyncBackend))
    .add_plugin(loading::LoadingPlugin)
    .add_plugin(player::PlayerPlugin)
    .add_system(bevy::window::close_on_esc)
    .run();
}

pub struct DebugStatePlugin;

impl Plugin for DebugStatePlugin {
    fn build(&self, app: &mut App) {
      for state in AppState::variants() {
        app
          .add_system(debug_state_enter.in_schedule(OnEnter(state)))
          .add_system(debug_state_update.in_set(OnUpdate(state)))
          .add_system(debug_state_exit.in_schedule(OnExit(state)));
      }
    }
}

fn debug_state_enter (state: Res<State<AppState>>) {
  println!("Entering: {:?}", state.0);
}

fn  debug_state_update (state: Res<State<AppState>>) {
  if(state.is_changed()) {
    println!("Running in: {:?}", state.0);
  }
}

fn debug_state_exit (state: Res<State<AppState>>) {
  println!("Exiting: {:?}", state.0);
}

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0;

// fn main() {
//   App::new()
//       .add_plugins(DefaultPlugins)
//       // this system will run once every update (it should match your screen's refresh rate)
//       .add_system(frame_update)
//       // add our system to the fixed timestep schedule
//       .add_system(fixed_update.in_schedule(CoreSchedule::FixedUpdate))
//       // configure our fixed timestep schedule to run twice a second
//       .insert_resource(FixedTime::new_from_secs(TIME_STEP))
//       .run();
// }