use bevy::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha12Rng;
use uuid::Uuid;

use crate::common::GameConfig;

#[derive(Clone, Debug, Copy)]
pub enum Action {
    Chilling,
    MovingAround,
    GoingForFood,
}

#[derive(Component, Clone, Debug, Copy)]
pub struct Cell {
    pub id: Uuid,
    pub pos_x: f32,
    pub pos_y: f32,
    pub width: f32,
    pub height: f32,
    pub movement_speed: f32,
    pub vision_range: f32,
    pub vision_angle: f32,
    pub health: f32,
    pub energy: f32,
    pub age: f32,
    pub target_location: Option<Vec2>,
    pub color: Color,
    pub rotation: f32,
    pub action: Action,
}

impl Cell {
    pub fn new(
        seeded_rng: &mut ChaCha12Rng,
        game_config: &GameConfig,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> (
        bevy::prelude::Mesh2d,
        bevy::prelude::MeshMaterial2d<ColorMaterial>,
        bevy::prelude::Transform,
        crate::common::Collider,
        Cell,
    ) {
        let x = seeded_rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0);
        let y = seeded_rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0);
        let width = seeded_rng.gen_range(15.0..25.0);
        let height = seeded_rng.gen_range(15.0..25.0);
        let color = Color::linear_rgb(
            seeded_rng.gen_range(0.0..1.0),
            seeded_rng.gen_range(0.0..1.0),
            seeded_rng.gen_range(0.0..1.0),
        );

        return (
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(color))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            crate::common::Collider,
            Cell {
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                width,
                height,
                health: 100.0,
                energy: 100.0,
                age: 0.0,
                movement_speed: seeded_rng.gen_range(15.0..100.0),
                vision_range: seeded_rng.gen_range(50.0..200.0),
                target_location: None,
                color,
                vision_angle: seeded_rng.gen_range(10.0..180.0),
                rotation: 0.0,
                action: Action::Chilling,
            },
        );
    }

    pub fn random_target(&mut self, seeded_rng: &mut ChaCha12Rng, game_config: &GameConfig) {
        self.target_location = Some(Vec2::new(
            seeded_rng.gen_range(-game_config.map_width / 2.0..game_config.map_width / 2.0),
            seeded_rng.gen_range(-game_config.map_height / 2.0..game_config.map_height / 2.0),
        ));

        self.action = Action::MovingAround;
    }

    pub fn draw_gismos(self, gizmos: &mut Gizmos) {
        gizmos.circle_2d(
            Vec2::new(self.pos_x, self.pos_y),
            self.vision_range,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        let forward = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(self.rotation));
        let half_angle = self.vision_angle.to_radians() / 2.0;
        let left = forward.rotate(Vec2::from_angle(-half_angle));
        let right = forward.rotate(Vec2::from_angle(half_angle));

        gizmos.line_2d(
            Vec2::new(self.pos_x, self.pos_y),
            Vec2::new(self.pos_x, self.pos_y) + left * self.vision_range,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );
        gizmos.line_2d(
            Vec2::new(self.pos_x, self.pos_y),
            Vec2::new(self.pos_x, self.pos_y) + right * self.vision_range,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        if let Some(location) = self.target_location {
            gizmos.circle_2d(location, 1.0, Color::linear_rgb(1.0, 0.0, 0.0));
        }
    }

    pub fn movement(&mut self, transform: &mut Transform, game_config: &GameConfig, time: f32) {
        if let Some(target) = self.target_location {
            let movement_speed = self.movement_speed * (self.energy / 100.0);
            let direction = (target - transform.translation.truncate()).normalize();
            let rotation =
                Quat::from_rotation_arc(Vec3::Y, Vec3::new(direction.x, direction.y, 0.0));

            transform.rotation = rotation;

            let dx = direction.x * movement_speed * time;
            let dy = direction.y * movement_speed * time;
            let nx = (transform.translation.x + dx)
                .clamp(-game_config.map_width / 2., game_config.map_width / 2.);
            let ny = (transform.translation.y + dy)
                .clamp(-game_config.map_height / 2., game_config.map_height / 2.);

            transform.translation.x = nx;
            self.pos_x = nx;

            transform.translation.y = ny;
            self.pos_y = ny;
            self.energy -= game_config.movement_cost;
            self.rotation = rotation.to_euler(EulerRot::XYZ).2
        }
    }
}

pub fn blend_colors(color1: LinearRgba, color2: LinearRgba, rng: &mut ChaCha12Rng) -> Color {
    let r = (color1.red + color2.red) / 2.0 * rng.gen_range(0.9..1.1);
    let g = (color1.green + color2.green) / 2.0 * rng.gen_range(0.9..1.1);
    let b = (color1.blue + color2.blue) / 2.0 * rng.gen_range(0.9..1.1);

    Color::linear_rgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}
