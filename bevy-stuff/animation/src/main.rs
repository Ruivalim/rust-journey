use bevy::prelude::*;
use helpers::camera::{CameraBehaviors, ZoomKeys};
use player::Player;

mod helpers;
mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                helpers::animation::execute_animations_system,
                helpers::camera::configure_camera_2d::<Player>(CameraBehaviors {
                    zoom_keys: Some(ZoomKeys {
                        zoom_in: KeyCode::KeyZ,
                        zoom_out: KeyCode::KeyX,
                        zoom_step: 0.2,
                        min_scale: 0.5,
                        max_scale: 2.0,
                    }),
                }),
            ),
        )
        .add_systems(Update, handle_input)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    let character_texture = asset_server.load("Player/Player.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 10, None, None);
    let texture_atlas = texture_atlas.add(layout);

    let animations = vec![
        helpers::animation::AnimationState::new("idle_down".to_string(), 0, 5),
        helpers::animation::AnimationState::new("idle_left".to_string(), 6, 11).flip_x(),
        helpers::animation::AnimationState::new("idle_right".to_string(), 6, 11),
        helpers::animation::AnimationState::new("idle_up".to_string(), 12, 17),
        helpers::animation::AnimationState::new("walk_down".to_string(), 18, 23),
        helpers::animation::AnimationState::new("walk_left".to_string(), 24, 29).flip_x(),
        helpers::animation::AnimationState::new("walk_right".to_string(), 24, 29),
        helpers::animation::AnimationState::new("walk_up".to_string(), 30, 35),
    ];

    let player_animation_config = helpers::animation::AnimationConfig::new(animations, 10);

    commands.spawn((
        Sprite {
            image: character_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas.clone(),
                index: player_animation_config.current_animation_first_sprite_index,
            }),
            flip_x: player_animation_config.current_animation_flip_x,
            flip_y: player_animation_config.current_animation_flip_y,
            ..default()
        },
        Transform::default()
            .with_translation(Vec3::new(0.0, 0.0, 1.0))
            .with_scale(Vec3::splat(5.)),
        player_animation_config,
        Player {
            position: Vec3::new(0.0, 0.0, 1.0),
        },
    ));
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    q_player: Single<
        (
            &mut helpers::animation::AnimationConfig,
            &mut Sprite,
            &mut Transform,
            &mut Player,
        ),
        With<Player>,
    >,
) {
    let mut player = q_player.into_inner();
    let velocity = 10.0;
    if keys.just_released(KeyCode::KeyW) {
        player.0.update_animation("idle_up".to_string());
    }
    if keys.just_released(KeyCode::KeyS) {
        player.0.update_animation("idle_down".to_string());
    }
    if keys.just_released(KeyCode::KeyA) {
        player.0.update_animation("idle_left".to_string());
    }
    if keys.just_released(KeyCode::KeyD) {
        player.0.update_animation("idle_right".to_string());
    }
    if keys.pressed(KeyCode::KeyW) {
        player.0.update_animation("walk_up".to_string());
        player.2.translation.y += velocity;
    }
    if keys.pressed(KeyCode::KeyS) {
        player.0.update_animation("walk_down".to_string());
        player.2.translation.y -= velocity;
    }
    if keys.pressed(KeyCode::KeyA) {
        player.0.update_animation("walk_left".to_string());
        player.2.translation.x -= velocity;
    }
    if keys.pressed(KeyCode::KeyD) {
        player.0.update_animation("walk_right".to_string());
        player.2.translation.x += velocity;
    }

    player.3.position = player.2.translation;
}
