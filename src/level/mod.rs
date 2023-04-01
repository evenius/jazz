use std::collections::HashSet;

use bevy::{prelude::*, app::PluginGroupBuilder};
use bevy_ecs_ldtk::prelude::*;

use crate::{loading::RegisterAssetEvent, AppState, player::{self, Player}, physics::ColliderBundle};

mod camera;

const TILE_SIZE: f32 = 32.;

#[derive(Copy, Clone, Debug, Component)]
pub struct Wall {
    pub width: f32,
    pub height: f32,
}


#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct WallCell;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: WallCell,
}

pub struct LevelPluginGroup;

impl PluginGroup for LevelPluginGroup {
  fn build(self) -> PluginGroupBuilder {
      PluginGroupBuilder::start::<Self>()
          .add(LdtkPlugin)
          .add(LevelPlugin)
          .add(camera::CameraPlugin)
  }
}


pub struct LevelPlugin;

impl Plugin for LevelPlugin {
 fn build(&self, app: &mut App) {
  app
    // Will add level 0 to the world
    .insert_resource(LevelSelection::Iid(String::from("9a124bc0-c640-11ed-ac82-5927f77176f9")))
    // Will load close neightbours of the level
    .insert_resource(LdtkSettings {
      level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
        load_level_neighbors: false,
      },
      set_clear_color: SetClearColor::FromLevelBackground,
      ..Default::default()
    })
    .add_system(load_level.in_schedule(OnEnter(AppState::GameLoading)))
    .add_system(spawn_level.in_schedule(OnExit(AppState::GameLoading)))
    .add_system(update_level_selection.in_set(OnUpdate(AppState::GameRunning)))
    .register_ldtk_int_cell::<WallBundle>(1)
    .register_ldtk_int_cell::<WallBundle>(2)
    // .register_ldtk_int_cell::<components::LadderBundle>(3)
    .register_ldtk_entity::<player::PlayerBundle>("Player");
    // .register_ldtk_entity::<components::MobBundle>("Mob")
    // .register_ldtk_entity::<components::ChestBundle>("Chest")
 }
}

fn load_level(asset_server: Res<AssetServer>, mut register_asset: EventWriter<RegisterAssetEvent>) {
  println!("Load level 'main'");
  register_asset.send(
    RegisterAssetEvent::new(asset_server.load_untyped("maps/main.ldtk"), "main.ldtk")
  );
}

fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn(LdtkWorldBundle {
      ldtk_handle: asset_server.load("maps/main.ldtk"),
      ..Default::default()
  });
}



pub fn update_level_selection(
  level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<Player>>,
  player_query: Query<(&Transform, &Worldly), With<Player>>,
  mut level_selection: ResMut<LevelSelection>,
  ldtk_levels: Res<Assets<LdtkLevel>>,
) {
  for (player_transform, wordly) in &player_query {


    for (level_handle, level_transform) in &level_query {
      if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
        let level_bounds = Rect {
          min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
          max: Vec2::new(
            level_transform.translation.x + ldtk_level.level.px_wid as f32,
            level_transform.translation.y + ldtk_level.level.px_hei as f32,
          ),
        };
              
              println!("player - x:{:?} y:{:?}", player_transform.translation.x, player_transform.translation.y.ceil());
              println!("in: - {:?}", wordly.entity_iid);
              println!("  {:?}",level_bounds.max.y);
              println!("___________");
              println!("|         |");
              println!("|         |");
              println!("{:?}         {:?}",level_bounds.min.x, level_bounds.max.x);
              println!("|         |");
              println!("___________");
              println!("  {:?}",level_bounds.min.y);

              if player_transform.translation.x < level_bounds.max.x
                  && player_transform.translation.x > level_bounds.min.x
                  && player_transform.translation.y < level_bounds.max.y
                  && player_transform.translation.y > level_bounds.min.y
                  && !level_selection.is_match(&0, &ldtk_level.level)
              {
                println!("SWITCH");
                  *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
              }
      }
  }
  }
}
