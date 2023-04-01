use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::physics::GroundDetection;

use super::{Player, PlayerState};


pub fn movement(
  input: Res<Input<KeyCode>>,
  time: Res<Time>,  
  mut query: Query<(&mut Velocity, &mut Player, &GroundDetection), With<Player>>,
) {
  for (mut velocity, mut player, ground_detection) in &mut query {
      if input.pressed(KeyCode::D) || input.pressed(KeyCode::A) {
          // D = <- , A = ->
          let direction = if input.pressed(KeyCode::D) { 1. } else { -1. };

          velocity.linvel.x = direction * 250.;
          
          player.facing = if direction > 0. { 1 } else { -1 };
          player.state = PlayerState::Walking;
      } else {
          player.state = PlayerState::Idle;
          let absolute_speed = velocity.linvel.x.abs();
          if absolute_speed > 0. {
              velocity.linvel.x = velocity.linvel.x * 0.09 * time.delta_seconds();
              if absolute_speed < 10.0 {
                  velocity.linvel.x = 0.0;
              }
          }
      }

      if input.just_pressed(KeyCode::Space) && (ground_detection.on_ground) {
        velocity.linvel.y = 500.;
      }
      
  }
}