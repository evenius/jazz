use bevy::{prelude::*, utils::{HashSet, HashMap}};
use bevy_ecs_ldtk::{EntityInstance, GridCoords, LdtkLevel, prelude::LayerInstance, IntGridCell, LdtkIntCell};
use bevy_rapier2d::prelude::*;

use crate::{level::{WallCell}, SHOW_COLLIDER_BOXES};



#[derive(Clone, Default, Component)]
pub struct GroundDetection {
    pub on_ground: bool,
}

#[derive(Component)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
    pub intersecting_ground_entities: HashSet<Entity>,
}

#[derive(Clone, Debug, Default, Bundle)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub damping: Damping,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct SensorBundle {
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub rotation_constraints: LockedAxes,
}

impl From<&EntityInstance> for ColliderBundle {
  fn from(entity_instance: &EntityInstance) -> ColliderBundle {
      let rotation_constraints = LockedAxes::ROTATION_LOCKED;

      match entity_instance.identifier.as_ref() {
         "Player" => ColliderBundle {
            collider: Collider::cuboid(16., 16.),
            // collider: Collider::cuboid(6., 14.),
            rigid_body: RigidBody::Dynamic,
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            rotation_constraints,
            ..Default::default()
        },
          // "Mob" => ColliderBundle {
          //     collider: Collider::cuboid(5., 5.),
          //     rigid_body: RigidBody::KinematicVelocityBased,
          //     rotation_constraints,
          //     ..Default::default()
          // },
          // "Chest" => ColliderBundle {
          //     collider: Collider::cuboid(8., 8.),
          //     rigid_body: RigidBody::Dynamic,
          //     rotation_constraints,
          //     gravity_scale: GravityScale(1.0),
          //     friction: Friction::new(0.5),
          //     density: ColliderMassProperties::Density(15.0),
          //     ..Default::default()
          // },
          other => {
            println!("No collider for {}, using default.", other);
            ColliderBundle::default()
          },
      }
  }
}

impl From<IntGridCell> for SensorBundle {
    fn from(int_grid_cell: IntGridCell) -> SensorBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        // ladder
        if int_grid_cell.value == 3 {
            SensorBundle {
                collider: Collider::cuboid(8., 8.),
                sensor: Sensor,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
            }
        } else {
            SensorBundle::default()
        }
    }
}


pub fn update_on_ground(
    mut ground_detectors: Query<&mut GroundDetection>,
    ground_sensors: Query<&GroundSensor, Changed<GroundSensor>>,
) {
    for sensor in &ground_sensors {
        if let Ok(mut ground_detection) = ground_detectors.get_mut(sensor.ground_detection_entity) {
            ground_detection.on_ground = !sensor.intersecting_ground_entities.is_empty();
        }
    }
}


pub fn ground_detection(
    mut ground_sensors: Query<&mut GroundSensor>,
    mut collisions: EventReader<CollisionEvent>,
    collidables: Query<With<Collider>, Without<Sensor>>,
) {
    for collision_event in collisions.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.insert(*e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.insert(*e2);
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.remove(e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.remove(e2);
                    }
                }
            }
        }
    }
}


pub fn spawn_ground_sensor(
  mut commands: Commands,
  detect_ground_for: Query<(Entity, &Collider), Added<GroundDetection>>,
) {
  for (entity, shape) in &detect_ground_for {
      if let Some(cuboid) = shape.as_cuboid() {
          let Vec2 {
              x: half_extents_x,
              y: half_extents_y,
          } = cuboid.half_extents();

          let detector_shape = Collider::cuboid(half_extents_x / 2.0, 2.);

          let sensor_translation = Vec3::new(0., -half_extents_y, 0.);


          commands.entity(entity).with_children(|builder| {
            if SHOW_COLLIDER_BOXES {
                let overlay_translation = Vec3::new(0., -half_extents_y, 100.);
                
                // Spawn a visual representation of the ground sensor
                builder.spawn(SpriteBundle { // Add semi-transparent blue box to visualize the sensor
                    sprite: Sprite {
                        color: Color::rgba(0.0, 0.0, 1.0, 0.5),
                        custom_size: Some(Vec2::new(half_extents_x, half_extents_y)),
                        ..default()
                    },
                    transform: Transform::from_translation(overlay_translation),
                    ..Default::default()
                });
            }
                
              builder
                  .spawn_empty()
                  .insert(ActiveEvents::COLLISION_EVENTS)
                  .insert(detector_shape)
                  .insert(Sensor)
                  .insert(Transform::from_translation(sensor_translation))
                  .insert(GlobalTransform::default())
                  .insert(GroundSensor {
                      ground_detection_entity: entity,
                      intersecting_ground_entities: HashSet::new(),
                  });
          });
      } else {
        println!("Cannot spawn ground sensor for {:?}", entity);
      }
  }
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<WallCell>>,
    parent_query: Query<&Parent, Without<WallCell>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {

                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        if SHOW_COLLIDER_BOXES {

                            level.spawn(SpriteBundle { // Add semi-transparent blue box to visualize the sensor
                                sprite: Sprite {
                                    color: Color::rgba(0.0, 1.0, 0., 0.5),
                                    custom_size: Some(Vec2::new((wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32,
                                    (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32)),
                                    ..default()
                                },
                                transform: Transform::from_xyz(
                                    (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                    (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                    100.,
                                ),
                                ..Default::default()
                            });
                        }

                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}
