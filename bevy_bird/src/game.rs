use std::ops::Range;

use bevy::{
    color::palettes::css::{PINK, YELLOW},
    input::common_conditions::input_just_released,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier2d::{
    plugin::RapierContext,
    prelude::{Collider, KinematicCharacterController, RigidBody},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

use crate::common::despawn_screen;
use crate::common::GameState;

#[derive(Resource)]
struct RandomSource(ChaCha12Rng);

#[derive(Component)]
struct Player {
    y: f32,
    movement_y: f32,
    gravity: f32,
}

#[derive(Resource)]
struct PlayerEntity(Entity);

struct Column {
    id_top: Entity,
    id_down: Entity,
    x: f32,
    height_top: f32,
    heigh_down: f32,
}

#[derive(Component)]
struct OnGameScreen;

#[derive(Resource)]
struct Columns(Vec<Column>);

#[derive(Component)]
struct ColumnMarker;

#[derive(Component)]
struct PointMarker;

#[derive(Resource)]
struct WindowSize {
    y_range: Range<f32>,
}

#[derive(Resource)]
struct PointMarkers(Vec<Entity>);

#[derive(Resource)]
struct Points(Vec<Entity>);

const COLUMN_WIDTH: f32 = 50.0;

pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), setup)
        .add_systems(
            Update,
            (
                back_to_menu
                    .run_if(input_just_released(KeyCode::Escape).and(in_state(GameState::Game))),
                input_handle
                    .run_if(input_just_released(KeyCode::Space).and(in_state(GameState::Game))),
                spawn_columns.run_if(in_state(GameState::Game)),
                point_update.run_if(in_state(GameState::Game).and(resource_changed::<Points>)),
            ),
        )
        .add_systems(
            FixedUpdate,
            (player_movement_handle, move_columns, handle_game_update)
                .chain()
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(OnExit(GameState::Game), despawn_screen::<OnGameScreen>);
}

fn setup(
    mut commands: Commands,
    q_window: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let player_handle = asset_server.load("textures/urubu.PNG");

    commands.remove_resource::<Columns>();
    commands.remove_resource::<RandomSource>();
    commands.remove_resource::<WindowSize>();
    commands.remove_resource::<PlayerEntity>();
    commands.remove_resource::<PointMarkers>();
    commands.remove_resource::<Points>();

    let window = q_window.single();
    let w_width = window.width();
    let w_height = window.height();
    let seeded_rng = ChaCha12Rng::from_entropy();

    let player_entity = commands
        .spawn((
            RigidBody::KinematicPositionBased,
            KinematicCharacterController::default(),
            Collider::cuboid(22.0, 22.0),
            Sprite::from_image(player_handle),
            Transform::default().with_translation(Vec3::new(
                (0.0 - (w_width / 2.0)) + 50.0 + 100.0,
                0.0,
                0.0,
            )),
            Player {
                gravity: -3.0,
                movement_y: 0.0,
                y: 0.0,
            },
            OnGameScreen,
        ))
        .id();
    let text_font = TextFont {
        font_size: 50.0,
        ..default()
    };

    commands.spawn((
        Text2d::new(format!("Points: 0")),
        text_font.clone(),
        TextColor(Color::WHITE),
        Transform::default().with_translation(Vec3::new(0.0, w_height / 2.0 - 50.0, 0.0)),
        OnGameScreen,
    ));

    commands.insert_resource(RandomSource(seeded_rng));
    commands.insert_resource(Columns(vec![]));
    commands.insert_resource(PointMarkers(vec![]));
    commands.insert_resource(WindowSize {
        y_range: -(w_height / 2.0)..(w_height / 2.0),
    });
    commands.insert_resource(PlayerEntity(player_entity));
    commands.insert_resource(Points(vec![]));
}

fn player_movement_handle(
    q_player: Single<(&mut Player, &mut Transform)>,
    mut controllers: Query<&mut KinematicCharacterController>,
) {
    let mut controller = controllers.single_mut();
    let mut player = q_player.into_inner();

    let total_movement = player.0.gravity + player.0.movement_y;
    controller.translation = Some(Vec2::new(0.0, total_movement));

    if total_movement > 0.0 {
        player.1.rotation = Quat::from_rotation_z(0.3);
    } else {
        player.1.rotation = Quat::from_rotation_z(-0.3);
    }

    player.0.movement_y = (player.0.movement_y - 1.0).clamp(0.0, player.0.movement_y);
    player.0.y = player.1.translation.y;
}

fn input_handle(keys: Res<ButtonInput<KeyCode>>, q_player: Single<&mut Player>) {
    let mut player = q_player.into_inner();
    if keys.just_released(KeyCode::Space) {
        player.movement_y = 15.0;
    }
}

fn back_to_menu(keys: Res<ButtonInput<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if keys.just_released(KeyCode::Escape) {
        game_state.set(GameState::Menu)
    }
}

fn spawn_columns(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut columns: ResMut<Columns>,
    mut point_markers: ResMut<PointMarkers>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut random_source: ResMut<RandomSource>,
) {
    let rng = &mut random_source.0;
    let window = q_window.single();
    let w_height = window.height();
    let count = columns.0.len() as i32;
    let gap = 150.0;
    let column_size = w_height - gap;
    let column_spaces = 500.0;

    for _ in 0..(20 - count) {
        let count = columns.0.len() as f32;
        let column_up_height = column_size - rng.gen_range(column_size / 2.0..column_size);
        let x = count * column_spaces;
        let column_up: Entity = commands
            .spawn((
                RigidBody::KinematicPositionBased,
                Collider::cuboid(COLUMN_WIDTH / 2.0, column_up_height / 2.0),
                Mesh2d(meshes.add(Rectangle::new(COLUMN_WIDTH, column_up_height))),
                MeshMaterial2d(materials.add(Color::from(PINK))),
                Transform::default().with_translation(Vec3::new(
                    x,
                    (w_height / 2.0) - (column_up_height / 2.0),
                    0.0,
                )),
                ColumnMarker,
                OnGameScreen,
            ))
            .id();

        let column_down_height = column_size - column_up_height;
        let column_down = commands
            .spawn((
                RigidBody::KinematicPositionBased,
                Collider::cuboid(COLUMN_WIDTH / 2.0, column_down_height / 2.0),
                Mesh2d(meshes.add(Rectangle::new(COLUMN_WIDTH, column_down_height))),
                MeshMaterial2d(materials.add(Color::from(PINK))),
                Transform::default().with_translation(Vec3::new(
                    x,
                    -((w_height / 2.0) - (column_down_height / 2.0)),
                    1.0,
                )),
                ColumnMarker,
                OnGameScreen,
            ))
            .id();

        columns.0.push(Column {
            id_top: column_up,
            id_down: column_down,
            x,
            height_top: column_up_height,
            heigh_down: column_down_height,
        });

        //0.0 - w_height / 2.0;

        let column_point = commands
            .spawn((
                RigidBody::KinematicPositionBased,
                Collider::cuboid(1.0, gap / 2.0),
                Transform::default().with_translation(Vec3::new(
                    x,
                    -((0.0 - w_height / 2.0) + column_up_height + gap / 2.0),
                    0.0,
                )),
                PointMarker,
                OnGameScreen,
            ))
            .id();

        point_markers.0.push(column_point);
    }
}

fn move_columns(
    mut columns: ResMut<Columns>,
    mut commands: Commands,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_columns: Query<&mut Transform, (With<ColumnMarker>, Without<PointMarker>)>,
    mut point_markers: ResMut<PointMarkers>,
    mut q_point_markers: Query<&mut Transform, (With<PointMarker>, Without<ColumnMarker>)>,
) {
    let window = q_window.single();
    let w_width = window.width() / 2.0;
    let velocity = 5.0;

    let mut new_columns = vec![];
    let mut new_markers = vec![];

    for point in point_markers.0.iter() {
        let mut transform = q_point_markers.get_mut(*point).ok().unwrap();
        let new_x = transform.translation.x - velocity;

        if new_x < -(w_width + 200.0) {
            commands.entity(*point).despawn();
        } else {
            transform.translation.x = new_x;
            new_markers.push(*point);
        }
    }

    for column in columns.0.iter_mut() {
        let top_column = column.id_top;
        let down_column = column.id_down;
        let new_x = column.x - velocity;

        if new_x < -(w_width + 200.0) {
            commands.entity(top_column).despawn();
            commands.entity(down_column).despawn();
        } else {
            if let Ok(mut transform) = q_columns.get_mut(top_column) {
                transform.translation.x = new_x;
            }
            if let Ok(mut transform) = q_columns.get_mut(down_column) {
                transform.translation.x = new_x;
            }

            new_columns.push(Column {
                id_top: top_column,
                id_down: down_column,
                x: new_x,
                height_top: column.height_top,
                heigh_down: column.heigh_down,
            });
        }
    }

    columns.0 = new_columns;
    point_markers.0 = new_markers;
}

fn handle_game_update(
    q_player: Single<&Player>,
    window_size: Res<WindowSize>,
    mut game_state: ResMut<NextState<GameState>>,
    rapier_context: Query<&RapierContext>,
    player_entity: Res<PlayerEntity>,
    columns: Res<Columns>,
    points_marker: Res<PointMarkers>,
    mut points: ResMut<Points>,
) {
    let rapier_context = rapier_context.single();
    let player = q_player.into_inner();

    if !window_size.y_range.contains(&player.y) {
        game_state.set(GameState::GameOver)
    }

    for contatcs in rapier_context.contact_pairs_with(player_entity.0) {
        for column in columns.0.iter() {
            if contatcs.collider2().eq(&column.id_top) || contatcs.collider2().eq(&column.id_down) {
                game_state.set(GameState::GameOver)
            }
        }
        for point in points_marker.0.iter() {
            if contatcs.collider2().eq(point) {
                if !points.0.contains(&contatcs.collider2()) {
                    points.0.push(contatcs.collider2());
                }
            }
        }
    }
}

fn point_update(points: ResMut<Points>, mut q_points: Query<&mut Text2d, With<OnGameScreen>>) {
    for mut text in q_points.iter_mut() {
        text.0 = format!("Points: {}", points.0.len());
    }
}
