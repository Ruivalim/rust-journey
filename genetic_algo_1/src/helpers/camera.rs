use bevy::{input::ButtonInput, prelude::*, render::camera::Camera};
#[allow(dead_code)]
pub struct ZoomKeys {
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub min_scale: f32,
    pub max_scale: f32,
    pub zoom_step: f32,
}
#[allow(dead_code)]
pub struct FollowingConfigs {
    pub fixed_at_object: bool,
    pub distance_of_movement_without_camera_movement_x: f32,
    pub distance_of_movement_without_camera_movement_y: f32,
}
#[allow(dead_code)]
impl FollowingConfigs {
    pub fn default() -> FollowingConfigs {
        return FollowingConfigs {
            fixed_at_object: true,
            distance_of_movement_without_camera_movement_x: 0.0,
            distance_of_movement_without_camera_movement_y: 0.0,
        };
    }

    pub fn easing() -> FollowingConfigs {
        return FollowingConfigs {
            fixed_at_object: false,
            distance_of_movement_without_camera_movement_x: 600.0,
            distance_of_movement_without_camera_movement_y: 250.0,
        };
    }
}
#[allow(dead_code)]
pub struct CameraBehaviors {
    pub zoom_keys: Option<ZoomKeys>,
    pub following_configs: FollowingConfigs,
}
#[allow(dead_code)]
fn signed_distance_to_range(num: f32, min: f32, max: f32) -> f32 {
    if num < min {
        min - num
    } else if num > max {
        max - num
    } else {
        0.0
    }
}
#[allow(dead_code)]
// This camera follows the player
pub fn configure_camera_2d<T: Component>(
    behaviors: CameraBehaviors,
) -> impl FnMut(
    Res<ButtonInput<KeyCode>>,
    Query<(&mut Transform, &mut OrthographicProjection), (With<Camera>, Without<T>)>,
    Query<&Transform, (With<T>, Without<Camera>)>,
) {
    move |keyboard_input, mut q_camera, q_player| {
        let player_transform = match q_player.get_single() {
            Ok(transform) => transform,
            Err(err) => {
                error!("Failed to get single player transform: {:?}", err);
                return;
            }
        };

        for (mut camera_transform, mut camera_ortho) in q_camera.iter_mut() {
            if let Some(zoom_keys) = &behaviors.zoom_keys {
                if keyboard_input.pressed(zoom_keys.zoom_in) {
                    camera_ortho.scale += zoom_keys.zoom_step;
                }

                if keyboard_input.pressed(zoom_keys.zoom_out) {
                    camera_ortho.scale -= zoom_keys.zoom_step;
                }

                camera_ortho.scale = camera_ortho
                    .scale
                    .clamp(zoom_keys.min_scale, zoom_keys.max_scale)
            }

            if behaviors.following_configs.fixed_at_object {
                camera_transform.translation = player_transform.translation;
            } else {
                let camera_x = camera_transform.translation.x;
                let camera_y = camera_transform.translation.y;
                let half_distance_x = behaviors
                    .following_configs
                    .distance_of_movement_without_camera_movement_x
                    * 0.5
                    * camera_ortho.scale;
                let half_distance_y = behaviors
                    .following_configs
                    .distance_of_movement_without_camera_movement_y
                    * 0.5
                    * camera_ortho.scale;

                let distance_x = signed_distance_to_range(
                    player_transform.translation.x,
                    camera_x - half_distance_x,
                    camera_x + half_distance_x,
                );

                let distance_y = signed_distance_to_range(
                    player_transform.translation.y,
                    camera_y - half_distance_y,
                    camera_y + half_distance_y,
                );

                const LERP_FACTOR: f32 = 0.1;

                if distance_x != 0.0 {
                    camera_transform.translation.x -= distance_x * LERP_FACTOR;
                }
                if distance_y != 0.0 {
                    camera_transform.translation.y -= distance_y * LERP_FACTOR;
                }
            }

            let z = camera_transform.translation.z;
            camera_transform.translation.z = z;
        }
    }
}

// This camera is independent
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyZ) {
            ortho.scale += 0.1;
        }

        if keyboard_input.pressed(KeyCode::KeyX) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < 0.5 {
            ortho.scale = 0.5;
        }

        let z = transform.translation.z;
        transform.translation += time.delta_secs() * direction * 500.;
        transform.translation.z = z;
    }
}
