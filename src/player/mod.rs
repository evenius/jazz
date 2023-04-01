use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use std::collections::HashMap;
use crate::SHOW_COLLIDER_BOXES;
use crate::physics::{ColliderBundle, GroundDetection};

use crate::{AppState, loading::RegisterAssetEvent};

use movement::movement;

mod movement;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player {
    pub facing: i8,
    pub state: PlayerState
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum PlayerState {
  #[default]
  Idle,
  Walking,
  // Jumping,
  // Shooting
}

#[derive(Resource, Default)]
pub struct PlayerSpriteHandles {
    handles: Vec<HandleUntyped>,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    pub player: Player,
    #[sprite_bundle("image/player.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    #[bundle]
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    pub ground_detection: GroundDetection,
    #[worldly]
    pub worldly: Worldly,
    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
 fn build(&self, app: &mut App) {
  app
    .init_resource::<PlayerSpriteHandles>()
    // .add_system(spawn_player.in_schedule(OnEnter(AppState::GameRunning)))
    .add_system(load_sprites.in_schedule(OnEnter(AppState::GameLoading)))
    .add_system(spawn_animated_player_sprites.in_set(OnUpdate(AppState::GameRunning)))
    .add_system(animate_sprite.in_set(OnUpdate(AppState::GameRunning)))
    .add_system(movement.in_set(OnUpdate(AppState::GameRunning)));
 }
}

fn load_sprites(mut commands: Commands, mut rpg_sprite_handles: ResMut<PlayerSpriteHandles>, asset_server: Res<AssetServer>, mut register_asset: EventWriter<RegisterAssetEvent>) {
    println!("Load level 'main'");
    register_asset.send(
      RegisterAssetEvent::new(asset_server.load_untyped("image/player.png"), "Player sprite")
    );

    let handles = asset_server.load_folder("image/adventurer").unwrap();

    handles.iter().for_each(|handle| {
        register_asset.send(
            RegisterAssetEvent::new(handle.clone(), asset_server.get_handle_path(handle).unwrap().path().to_str().unwrap())
        );
    });

    rpg_sprite_handles.handles = handles;
}

#[derive(Component)]
struct AnimationIndices {
  pub indices: HashMap<&'static str, Vec<usize>>,
  pub current_index: usize
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

// , Without<TextureAtlasSprite>
pub fn spawn_animated_player_sprites(mut commands: Commands, asset_server: Res<AssetServer>, rpg_sprite_handles: Res<PlayerSpriteHandles>, mut texture_atlases: ResMut<Assets<TextureAtlas>>, mut textures: ResMut<Assets<Image>>,  player_query: Query<Entity, (With<Player>, With<Sprite>, Without<TextureAtlasSprite>)>) {

    if let Ok(player_entity) = player_query.get_single()  {
        let mut texture_atlas_builder = TextureAtlasBuilder::default();

        for handle in &rpg_sprite_handles.handles {
            let handle = handle.typed_weak();
            let Some(texture) = textures.get(&handle) else {
                warn!("{:?} did not resolve to an `Image` asset.", asset_server.get_handle_path(handle));
                continue;
            };

            texture_atlas_builder.add_texture(handle, texture);
        }
        
        let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();

        // show how many textures are loaded:
        println!("Loaded {} textures", rpg_sprite_handles.handles.len());
        
        let ( mut idles, mut walking ) = rpg_sprite_handles.handles
            .iter()
            .fold(( vec![], vec![] ), |(mut idles, mut walking), handle| {
                if let Some(asset_path) = asset_server.get_handle_path(handle) {
                    let path = asset_path.path().to_str().unwrap().to_owned();
                    // let pathstring = path.to_string_lossy();
                    if path.contains("adventurer-idle-2") {
                        idles.push((path, handle.clone()));
                    } else if path.contains("adventurer-run") {
                        walking.push((path, handle.clone()));
                    }
            }
            (idles, walking)
        });

            idles.sort_by_key(|(p, _)| p.to_owned());
            walking.sort_by_key(|(p, _)| p.to_owned());

            let idle_indices: Vec<usize> = idles.iter().map(| (path, handle) | {
            let pathstring = path;
            println!("Path: {}", pathstring);
            texture_atlas.get_texture_index(
                &handle.clone_weak().typed::<Image>()
            ).unwrap()
            }).collect();

            let walking_indices = walking.iter().map(| (path, handle) | {
            let pathstring = path;
            println!("Path: {}", pathstring);
            texture_atlas.get_texture_index(
                &handle.clone_weak().typed::<Image>()
            ).unwrap()
            }).collect();

            let first_idle = idle_indices[0];

            let hash_map = HashMap::from([
            ("idle", idle_indices),
            ("walk", walking_indices)
            ]);

            let animation_indices = AnimationIndices {
                indices: hash_map,
                current_index: 0,
            };

            let atlas_handle = texture_atlases.add(texture_atlas);
            commands.entity(player_entity)
                .remove::<(Sprite, Handle<Image>)>()
                .insert( (
                    TextureAtlasSprite::new(first_idle),
                    atlas_handle,
                    // transform: Transform::from_xyz(100., 0., 0.),
                    animation_indices,
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                ));
                if SHOW_COLLIDER_BOXES { 
                    commands.entity(player_entity).with_children(|player| {
                        player.spawn(SpriteBundle { // Add semi-transparent blue box to visualize the sensor
                            sprite: Sprite {
                                color: Color::rgba(1.0, 0., 0., 0.5),
                                custom_size: Some(Vec2::new(32., 32.)),
                                ..default()
                            },
                            ..default()
                        });
                    });
                }
    }
}
    

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &Player,
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
  ) {
    for (player, mut indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
  
          let active_index_key = if player.state == PlayerState::Walking {
              "walk"
          } else {
              "idle"
          };
  
          let active_index_vec = indices.indices.get(active_index_key).unwrap_or_else( || panic!("No indices found for key: {} in {:?}", active_index_key, indices.indices));
  
          let index_len = active_index_vec.len().clone();
  
          let current_index = &mut indices.current_index;
  
          indices.current_index = if *current_index >= index_len - 1 {
            0
          } else {
            *current_index + 1
          };
  
          let next_index = indices.current_index.clone();
          let next_sprite_index = indices.indices.get(active_index_key).unwrap().get(next_index).unwrap().clone();
  
          sprite.index = next_sprite_index;   
          sprite.flip_x = player.facing == -1;
          
        }
    }
  }