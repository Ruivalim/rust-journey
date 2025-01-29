use bevy::color::palettes::css::{BLUE, PURPLE, RED, YELLOW};
use bevy::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha12Rng;
use uuid::Uuid;

use crate::common::{self, GameConfig};
use crate::food;
use crate::helpers::neural_network::NeuralNetwork;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Action {
    MovingAround,
    GoingForFood,
    ////Duplicate,
}

#[derive(Clone, Debug)]
pub struct Genes {
    pub movement_speed: f32,
    pub vision_range: f32,
    pub vision_angle: f32,
    pub color: Color,
    pub metabolism: f32,

    pub energy_weight: f32,
    pub random_weight: f32,
}

#[derive(Component, Clone, Debug)]
pub struct Cell {
    pub health: f32,
    pub energy: f32,
    pub age: i32,
    pub fitness: f32,
    pub generation: i32,
    pub id: Uuid,
    pub pos_x: f32,
    pub pos_y: f32,
    pub target_location: Option<Vec2>,
    pub rotation: f32,
    pub action: Action,
    pub brain: NeuralNetwork,
    pub genes: Genes,
    pub offsprings_count: i32,
}

#[allow(dead_code)]
impl Cell {
    pub fn new(
        seeded_rng: &mut ChaCha12Rng,
        game_config: &GameConfig,
        cell_mesh: &Handle<Mesh>,
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
        let color = Color::linear_rgb(
            seeded_rng.gen_range(0.0..1.0),
            seeded_rng.gen_range(0.0..1.0),
            seeded_rng.gen_range(0.0..1.0),
        );

        let genes = Genes {
            movement_speed: seeded_rng.gen_range(15.0..100.0),
            vision_range: seeded_rng.gen_range(100.0..400.0),
            color,
            vision_angle: seeded_rng.gen_range(10.0..180.0),
            metabolism: seeded_rng.gen_range(0.5..5.0),
            energy_weight: seeded_rng.gen_range(0.9..1.2),
            random_weight: seeded_rng.gen_range(0.5..1.5),
        };

        return (
            Mesh2d(cell_mesh.clone()),
            MeshMaterial2d(materials.add(Color::from(color))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            crate::common::Collider,
            Cell {
                fitness: 0.0,
                brain: NeuralNetwork::new(6, 10, 3),
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                generation: 0,
                health: 100.0,
                energy: 50.0,
                age: 1,
                target_location: None,
                rotation: 0.0,
                action: Action::MovingAround,
                genes,
                offsprings_count: 0,
            },
        );
    }

    pub fn random_target(&mut self, seeded_rng: &mut ChaCha12Rng, game_config: &GameConfig) {
        let distance = seeded_rng.gen_range(0.0..self.genes.vision_range);

        // Generate a random angle within the vision cone
        let half_vision_angle = self.genes.vision_angle / 2.0;
        let random_angle = seeded_rng
            .gen_range(-half_vision_angle..half_vision_angle)
            .to_radians();

        let forward_vector = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(self.rotation));

        let target_offset = forward_vector.rotate(Vec2::from_angle(random_angle)) * distance;
        let target_position = Vec2::new(self.pos_x, self.pos_y) + target_offset;

        let clamped_target_position = Vec2::new(
            target_position
                .x
                .clamp(-game_config.map_width / 2.0, game_config.map_width / 2.0),
            target_position
                .y
                .clamp(-game_config.map_height / 2.0, game_config.map_height / 2.0),
        );

        self.target_location = Some(clamped_target_position);
    }

    pub fn draw_vision(
        &self,
        gizmos: &mut Gizmos,
        all_cells: &Vec<Cell>,
        all_foods: &Vec<food::Food>,
    ) {
        let center = Vec2::new(self.pos_x, self.pos_y);
        let radius = self.genes.vision_range;
        let rotation = self.rotation;
        let half_angle = self.genes.vision_angle.to_radians() / 2.0;

        let domain = Interval::UNIT;
        let curve = FunctionCurve::new(domain, |t| {
            let angle = -half_angle + t * 2.0 * half_angle;
            let direction = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(rotation + angle));
            center + direction * radius
        });

        gizmos.curve_2d(
            curve,
            (0..=100).map(|n| n as f32 / 100.0),
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        let left = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(rotation - half_angle));
        let right = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(rotation + half_angle));

        gizmos.line_2d(
            center,
            center + left * radius,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );
        gizmos.line_2d(
            center,
            center + right * radius,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        if let Some(location) = self.target_location {
            gizmos.circle_2d(location, 1.0, Color::linear_rgb(1.0, 0.0, 0.0));
        }

        for cell in all_cells {
            let cell_position = Vec2::new(cell.pos_x, cell.pos_y);
            if self.is_within_vision_cone(cell_position) {
                gizmos.circle_2d(cell_position, 20.0, self.genes.color);
            }
        }
        for food in all_foods {
            let food_position = Vec2::new(food.pos_x, food.pos_y);
            if self.is_within_vision_cone(food_position) {
                gizmos.circle_2d(food_position, 20.0, self.genes.color);
            }
        }
    }

    pub fn movement(&mut self, transform: &mut Transform, game_config: &GameConfig, time: f32) {
        if let Some(target) = self.target_location {
            let mut movement_speed = self.genes.movement_speed;
            if self.energy < 30.0 {
                movement_speed *= 0.5;
            }

            let direction = (target - transform.translation.truncate()).normalize_or_zero();

            if direction.is_nan() {
                self.target_location = None;
                return;
            }

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
            self.rotation = rotation.to_euler(EulerRot::XYZ).2;
        }
    }

    pub fn process_metabolism(&mut self, time: f32) {
        let base_rate = self.genes.metabolism;

        let activity_factor = match self.action {
            Action::MovingAround => 1.0,
            Action::GoingForFood => 1.2,
            //Action::Duplicate => 2.0,
        };

        let metabolism_rate = base_rate * activity_factor;

        self.energy = (self.energy - metabolism_rate * time).clamp(0.0, 100.0);

        if self.energy <= 0.0 {
            self.health = (self.health - (metabolism_rate * time)).clamp(0.0, 100.0);
        }
    }

    pub fn process_brain(
        &mut self,
        rng: &mut ChaCha12Rng,
        all_cells: &Vec<Cell>,
        all_foods: &Vec<food::Food>,
    ) {
        let energy_score = (self.energy.clamp(0.0001, 100.0) / 100.0);
        let low_energy = if self.energy < 30.0 { 1.0 } else { 0.0 };
        let is_healthy = if self.health > 70.0 { 1.0 } else { 0.0 };
        let random_score = self.genes.random_weight * rng.gen_range(0.0..0.1);

        let inputs = ndarray::array![
            self.health / 100.0,
            energy_score,
            low_energy,
            is_healthy,
            self.genes.metabolism / 100.0,
            random_score,
        ];

        let outputs = self.brain.feedforward(&inputs);
        let action_index = outputs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();

        self.action = match action_index.0 {
            0 => Action::MovingAround,
            1 => Action::GoingForFood,
            //2 => Action::Duplicate,
            _ => Action::MovingAround,
        };
    }

    pub fn create_offspring(
        &mut self,
        seeded_rng: &mut ChaCha12Rng,
        game_config: &GameConfig,
        cell_mesh: &Handle<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> (
        bevy::prelude::Mesh2d,
        bevy::prelude::MeshMaterial2d<ColorMaterial>,
        bevy::prelude::Transform,
        crate::common::Collider,
        Cell,
    ) {
        self.energy -= 50.0;
        self.offsprings_count += 1;

        let mut offspring_network = self.brain.clone();
        Cell::mutate(&mut offspring_network, game_config);

        let genes = Genes {
            movement_speed: mutate_gene(
                self.genes.movement_speed,
                seeded_rng,
                game_config.mutation_rate,
                15.0,
                100.0,
            ),
            vision_range: mutate_gene(
                self.genes.vision_range,
                seeded_rng,
                game_config.mutation_rate,
                100.0,
                400.0,
            ),
            vision_angle: mutate_gene(
                self.genes.vision_angle,
                seeded_rng,
                game_config.mutation_rate,
                10.0,
                180.0,
            ),
            metabolism: mutate_gene(
                self.genes.metabolism,
                seeded_rng,
                game_config.mutation_rate,
                0.5,
                5.0,
            ),
            color: self.genes.color,
            energy_weight: mutate_gene(
                self.genes.energy_weight,
                seeded_rng,
                game_config.mutation_rate,
                0.9,
                1.2,
            ),
            random_weight: mutate_gene(
                self.genes.random_weight,
                seeded_rng,
                game_config.mutation_rate,
                0.5,
                1.5,
            ),
        };

        let x = self.pos_x;
        let y = self.pos_y;

        return (
            Mesh2d(cell_mesh.clone()),
            MeshMaterial2d(materials.add(Color::from(genes.color))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            crate::common::Collider,
            Cell {
                fitness: 0.0,
                brain: offspring_network,
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                health: 100.0,
                energy: 50.0,
                age: 1,
                target_location: None,
                rotation: 0.0,
                action: Action::MovingAround,
                generation: self.generation + 1,
                genes,
                offsprings_count: 0,
            },
        );
    }

    pub fn is_within_vision_cone(&self, target_position: Vec2) -> bool {
        let cell_position = Vec2::new(self.pos_x, self.pos_y);

        let distance = cell_position.distance(target_position);
        if distance > self.genes.vision_range {
            return false;
        }

        let cell_forward = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(self.rotation));

        let to_target = (target_position - cell_position).normalize();

        let angle_to_target = cell_forward.angle_to(to_target).to_degrees();

        angle_to_target.abs() <= self.genes.vision_angle / 2.0
    }

    pub fn eat(&mut self, food: &food::Food) {
        self.energy = (self.energy + 15.0).clamp(0.0, 100.0);
        self.health = (self.energy + 15.0).clamp(0.0, 100.0);
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

fn age_function(age: f32) -> f32 {
    ((f32::cos((age / 13.0) - 22.0)) + 1.0) / 2.0
}

fn mutate_gene(value: f32, rng: &mut ChaCha12Rng, mutation_rate: f32, min: f32, max: f32) -> f32 {
    let mutation = rng.gen_range(-mutation_rate..mutation_rate) * value;
    (value + mutation).clamp(min, max)
}
