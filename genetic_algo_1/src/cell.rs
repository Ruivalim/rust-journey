use bevy::color::palettes::css::{BLUE, PURPLE, RED, YELLOW};
use bevy::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha12Rng;
use uuid::Uuid;

use crate::common::GameConfig;
use crate::helpers::neural_network::NeuralNetwork;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Action {
    Chilling,
    MovingAround,
    GoingForFood,
    FindMate,
}

#[derive(Component, Clone, Debug)]
pub struct Cell {
    pub fitness: f32,
    pub generation: i32,
    pub brain: NeuralNetwork,
    pub id: Uuid,
    pub pos_x: f32,
    pub pos_y: f32,
    pub width: f32,
    pub height: f32,
    pub movement_speed: f32,
    pub vision_range: f32,
    pub vision_angle: f32,
    pub health: f32,
    pub hunger: f32,
    pub energy: f32,
    pub age: i32,
    pub reproduction_urge: bool,
    pub target_location: Option<Vec2>,
    pub color: Color,
    pub rotation: f32,
    pub action: Action,
}

#[allow(dead_code)]
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
                fitness: 0.0,
                brain: NeuralNetwork::new(8, 10, 4),
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                generation: 0,
                width,
                hunger: 0.0,
                height,
                health: 100.0,
                energy: 100.0,
                age: 0,
                reproduction_urge: false,
                movement_speed: seeded_rng.gen_range(15.0..100.0),
                vision_range: seeded_rng.gen_range(100.0..400.0),
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

    pub fn draw_vision(&self, gizmos: &mut Gizmos) {
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

    pub fn draw_gizmos(&self, gizmos: &mut Gizmos) {
        match self.action {
            Action::Chilling => gizmos.circle_2d(Vec2::new(self.pos_x, self.pos_y), 10.0, BLUE),
            Action::GoingForFood => gizmos.circle_2d(Vec2::new(self.pos_x, self.pos_y), 10.0, RED),
            Action::MovingAround => {
                gizmos.circle_2d(Vec2::new(self.pos_x, self.pos_y), 10.0, YELLOW)
            }
            Action::FindMate => gizmos.circle_2d(Vec2::new(self.pos_x, self.pos_y), 10.0, PURPLE),
        };
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
            self.energy = (self.energy - game_config.movement_cost).clamp(0.0, 100.0);
            self.rotation = rotation.to_euler(EulerRot::XYZ).2
        }
    }

    pub fn process_brain(&mut self, rng: &mut ChaCha12Rng, game_config: &GameConfig) {
        self.hunger = (self.hunger + game_config.hunger_over_time).clamp(0.0, 100.0);

        if self.hunger == 100.0 {
            self.health = (self.health - game_config.life_lost_on_hungry).clamp(0.0, 100.0);
        }

        let low_energy = if self.energy < 30.0 { 1.0 } else { 0.0 };
        let high_hunger = if self.hunger > 70.0 { 1.0 } else { 0.0 };
        let is_healthy = if self.health > 70.0 { 1.0 } else { 0.0 };
        let has_urge_to_reproduce = if self.reproduction_urge { 1.0 } else { 0.0 };

        let inputs = ndarray::array![
            100.0 / self.hunger,
            self.health / 100.0,
            self.energy / 100.0,
            low_energy,
            high_hunger,
            is_healthy,
            has_urge_to_reproduce,
            rng.gen_range(0.0..1.0),
        ];

        let outputs = self.brain.feedforward(&inputs);
        let action_index = outputs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;

        self.action = match action_index {
            0 => Action::Chilling,
            1 => Action::MovingAround,
            2 => Action::GoingForFood,
            3 => {
                if self.reproduction_urge {
                    Action::FindMate
                } else {
                    Action::Chilling
                }
            }
            _ => Action::Chilling,
        };
    }

    fn find_closest_food(
        &self,
        transform: &Transform,
        food_query: &mut Query<
            (&mut Transform, &mut crate::food::Food, Entity),
            (With<crate::food::Food>, Without<Cell>),
        >,
    ) -> Option<Vec2> {
        food_query
            .iter()
            .map(|food_transform| food_transform.0.translation.truncate())
            .min_by(|a, b| {
                transform
                    .translation
                    .truncate()
                    .distance(*a)
                    .partial_cmp(&transform.translation.truncate().distance(*b))
                    .unwrap()
            })
    }

    pub fn create_offspring(
        seeded_rng: &mut ChaCha12Rng,
        game_config: &GameConfig,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        parent1: Cell,
        parent2: Cell,
    ) -> (
        bevy::prelude::Mesh2d,
        bevy::prelude::MeshMaterial2d<ColorMaterial>,
        bevy::prelude::Transform,
        crate::common::Collider,
        Cell,
    ) {
        let mut offspring_network = NeuralNetwork::crossover(&parent1.brain, &parent2.brain);
        Cell::mutate(&mut offspring_network, game_config);

        let width = blend(
            parent1.width,
            parent2.width,
            seeded_rng,
            game_config.mutation_rate,
        );
        let height = blend(
            parent1.height,
            parent2.height,
            seeded_rng,
            game_config.mutation_rate,
        );
        let movement_speed = blend(
            parent1.movement_speed,
            parent2.movement_speed,
            seeded_rng,
            game_config.mutation_rate,
        );
        let vision_range = blend(
            parent1.vision_range,
            parent2.vision_range,
            seeded_rng,
            game_config.mutation_rate,
        );
        let vision_angle = blend(
            parent1.vision_angle,
            parent2.vision_angle,
            seeded_rng,
            game_config.mutation_rate,
        );
        let color = blend_colors(
            parent1.color.to_linear(),
            parent2.color.to_linear(),
            seeded_rng,
        );

        let mut highest_generation = parent1.generation;

        if parent2.generation > highest_generation {
            highest_generation = parent2.generation;
        }

        let x = parent1.pos_x;
        let y = parent1.pos_y;

        return (
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(color))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            crate::common::Collider,
            Cell {
                fitness: 0.0,
                brain: NeuralNetwork::new(8, 10, 4),
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                width,
                hunger: 0.0,
                height,
                health: 100.0,
                energy: 100.0,
                age: 0,
                reproduction_urge: false,
                movement_speed,
                vision_range,
                target_location: None,
                color,
                vision_angle,
                rotation: 0.0,
                action: Action::Chilling,
                generation: highest_generation + 1,
            },
        );
    }

    pub fn mutate(network: &mut NeuralNetwork, game_config: &GameConfig) {
        let mut rng = rand::thread_rng();
        for weight in network.weights_input_hidden.iter_mut() {
            if rng.gen::<f32>() < game_config.mutation_rate {
                *weight += rng.gen_range(-0.1..0.1);
            }
        }
        for weight in network.weights_hidden_output.iter_mut() {
            if rng.gen::<f32>() < game_config.mutation_rate {
                *weight += rng.gen_range(-0.1..0.1);
            }
        }
        for bias in network.biases_hidden.iter_mut() {
            if rng.gen::<f32>() < game_config.mutation_rate {
                *bias += rng.gen_range(-0.1..0.1);
            }
        }
        for bias in network.biases_output.iter_mut() {
            if rng.gen::<f32>() < game_config.mutation_rate {
                *bias += rng.gen_range(-0.1..0.1);
            }
        }
    }
}

pub fn blend_colors(color1: LinearRgba, color2: LinearRgba, rng: &mut ChaCha12Rng) -> Color {
    let r = (color1.red + color2.red) / 2.0 * rng.gen_range(0.9..1.1);
    let g = (color1.green + color2.green) / 2.0 * rng.gen_range(0.9..1.1);
    let b = (color1.blue + color2.blue) / 2.0 * rng.gen_range(0.9..1.1);

    Color::linear_rgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

pub fn blend(value1: f32, value2: f32, seeded_rng: &mut ChaCha12Rng, mutation_rate: f32) -> f32 {
    let average = (value1 + value2) / 2.0;
    average + seeded_rng.gen_range(-mutation_rate..mutation_rate) * average
}
