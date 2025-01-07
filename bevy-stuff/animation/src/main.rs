use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use helpers::camera::{CameraBehaviors, FollowingConfigs, ZoomKeys};
use player::Player;

mod helpers;
mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
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
                    following_configs: FollowingConfigs::easing(),
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
            velocity: 10.0,
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
    let base_velocity = player.3.velocity;

    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();

        if direction.y.abs() > direction.x.abs() {
            if direction.y > 0.0 {
                player.0.update_animation("walk_up".to_string());
            } else {
                player.0.update_animation("walk_down".to_string());
            }
        } else {
            if direction.x > 0.0 {
                player.0.update_animation("walk_right".to_string());
            } else {
                player.0.update_animation("walk_left".to_string());
            }
        }

        player.2.translation.x += direction.x * base_velocity;
        player.2.translation.y += direction.y * base_velocity;
    } else {
        let current_animation = player.0.current_animation_name.as_str();
        let idle_animation = match current_animation {
            "walk_up" => "idle_up",
            "walk_down" => "idle_down",
            "walk_left" => "idle_left",
            "walk_right" => "idle_right",
            _ => "idle_down",
        };
        player.0.update_animation(idle_animation.to_string());
    }

    player.3.position = player.2.translation;
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize { x: 100, y: 100 };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(0),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0)
            .with_scale(Vec3::splat(5.)),
        ..Default::default()
    });
}
