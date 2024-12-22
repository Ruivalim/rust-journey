use bevy::{input::ButtonInput, prelude::*, render::camera::Camera};

pub struct ZoomKeys {
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub min_scale: f32,
    pub max_scale: f32,
    pub zoom_step: f32,
}

pub struct CameraBehaviors {
    pub zoom_keys: Option<ZoomKeys>,
}

pub fn configure_camera_2d<T: Component>(
    behaviors: CameraBehaviors,
) -> impl FnMut(
    Res<ButtonInput<KeyCode>>,
    Query<(&mut Transform, &mut OrthographicProjection), (With<Camera>, Without<T>)>,
    Query<&Transform, (With<T>, Without<Camera>)>,
) {
    move |keyboard_input, mut query, q_player| {
        let player_transform = match q_player.get_single() {
            Ok(transform) => transform,
            Err(err) => {
                error!("Failed to get single player transform: {:?}", err);
                return;
            }
        };

        for (mut transform, mut ortho) in query.iter_mut() {
            if let Some(zoom_keys) = &behaviors.zoom_keys {
                if keyboard_input.pressed(zoom_keys.zoom_in) {
                    ortho.scale += zoom_keys.zoom_step;
                }

                if keyboard_input.pressed(zoom_keys.zoom_out) {
                    ortho.scale -= zoom_keys.zoom_step;
                }

                ortho.scale = ortho.scale.clamp(zoom_keys.min_scale, zoom_keys.max_scale)
            }

            let z = transform.translation.z;
            transform.translation = player_transform.translation;
            transform.translation.z = z;
        }
    }
}
