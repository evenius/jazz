

use bevy::{prelude::*, asset::AssetPath};
use bevy_ecs_ldtk::LdtkAsset;

use crate::AppState;

#[derive(Resource)]
struct AssetsLoading {
    assets: Vec<HandleUntyped>,
    did_load_level: bool
}

impl Default for AssetsLoading {
    fn default() -> Self {
      Self {
        assets: Vec::new(),
        did_load_level: false
      }
    }
  }

// marker for Loading Screen stuff
#[derive(Component)]
struct LoadingScreenComponent;

#[derive(Component, Deref, DerefMut)]
struct LoadingTimer(Timer);

pub struct LoadingPlugin;

const LOADING_TIME: f32 = 10.0; // Seconds

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<RegisterAssetEvent>()
            .init_resource::<AssetsLoading>()
            .add_system(create_loading_screen.in_schedule(OnEnter(AppState::GameLoading)))
            .add_system(register_asset_loading.in_set(OnUpdate(AppState::GameLoading)))
            .add_system(register_level_loading.in_set(OnUpdate(AppState::GameLoading)))
            .add_system(check_images_ready.in_set(OnUpdate(AppState::GameLoading)))
            .add_system(show_loading_screen.in_set(OnUpdate(AppState::GameLoading)))
            .add_system(destroy_loading_screen.in_schedule(OnExit(AppState::GameLoading)))
            .add_system(debug_event_too_late.in_set(OnUpdate(AppState::GameRunning)))
            .add_system(destroy_loading_state.in_schedule(OnEnter(AppState::GameRunning)));
    }
}


fn create_loading_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let camera = Camera2dBundle::default();

    commands.insert_resource(AssetsLoading::default());

    commands.spawn((camera, LoadingScreenComponent));
    commands.spawn(
        (LoadingTimer(Timer::from_seconds(LOADING_TIME, TimerMode::Once)), LoadingScreenComponent)
    );

    commands.spawn((NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: BackgroundColor(Color::WHITE),
        ..default()
    }, LoadingScreenComponent))
    .with_children(|root| {
        // Text where we display current resolution
        root.spawn(TextBundle {
            text: Text::from_section("Loading", TextStyle {
                font: asset_server.load("fonts/roboto.ttf"),
                font_size: 64.0,
                color: Color::BLACK,
            }).with_alignment(TextAlignment::Center),
            ..default()        
        });
    });
    // state.set(AppState::LoadingGame).unwrap();
}

fn destroy_loading_state(
    mut commands: Commands
) {
    println!("Destroying loading state");
    // commands.remove_resource::<AssetsLoading>();
    // commands.init_resource::<AssetsLoading>();
}

fn show_loading_screen ( time: Res<Time>, mut next_state: ResMut<NextState<AppState>>, mut query: Query<&mut LoadingTimer>) {
    let mut timer = query.single_mut();
    timer.0.tick(time.delta());

    if timer.0.finished() {
        println!("Loading finished");
        next_state.set(AppState::GameRunning);
    }
}

fn destroy_loading_screen (mut commands: Commands, query: Query<Entity, With<LoadingScreenComponent>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}


// struct RegisterLDTKAssetEvent(Handle<LdtkAsset>);
pub struct RegisterAssetEvent {
    handle:  HandleUntyped,
    label: Option<String>
}

impl RegisterAssetEvent {
    pub fn new<'a, MaybeLabel>(handle: HandleUntyped, label: MaybeLabel) -> Self 
    where MaybeLabel: Into<Option<&'a str>> {
        Self {
            handle,
            label: label.into().map(|s| s.to_string())
        }
    }
}

fn register_asset_loading(mut ev_reg_asset: EventReader<RegisterAssetEvent>, mut loading: ResMut<AssetsLoading>) {
    for ev in ev_reg_asset.iter() {
        println!("Registering asset: {:?}", ev.label);
        loading.assets.push(ev.handle.clone());
    }
}


// Current automagically registerign ldtk assets
fn register_level_loading(
    mut ev_asset: EventReader<AssetEvent<LdtkAsset>>,
    assets: Res<Assets<LdtkAsset>>,
    mut loading: ResMut<AssetsLoading>,
  ) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let texture = assets.get(handle).unwrap();
                println!("LDTK loaded: {:?} (464870)",  texture.project.app_build_id);
                
                for (_, img_handle) in texture.tileset_map.iter() {
                    loading.assets.push(img_handle.clone_untyped());
                }
                loading.did_load_level = true;
            }
            AssetEvent::Modified { handle } => {
                // an image was modified
            }
            AssetEvent::Removed { handle } => {
                // an image was unloaded
            }
        }
    }
  }

  fn debug_event_too_late(mut ev_reg_image: EventReader<RegisterAssetEvent>, mut loading: ResMut<AssetsLoading>) {
    for ev in ev_reg_image.iter() {
        println!("Event too late: {:?}", ev.label);
    }
  }

fn check_images_ready(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut next_state: ResMut<NextState<AppState>>,
  ) {
    use bevy::asset::LoadState;

  
    match server.get_group_load_state(loading.assets.iter().map(|h| h.id())) {
        LoadState::Failed => {
            println!("Failed to load assets");
            // Just printing failed assets
            for handle in loading.assets.iter() {
                match server.get_load_state(handle.id()) {
                    LoadState::Failed => {
                        println!("Failed to load asset: {:?}", server.get_handle_path(handle));
                    }
                    LoadState::Loaded => {
                        println!("Succeeded: {:?}", server.get_handle_path(handle));
                    }
                    _ => {
                        println!("Loading: {:?}", server.get_handle_path(handle));
                    }
                }
            }
        }
        LoadState::Loaded => {
            if(loading.did_load_level) {
                next_state.set(AppState::GameRunning);
            }
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
  }