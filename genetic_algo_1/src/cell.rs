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
    Chilling,
    MovingAround,
    GoingForFood,
    FindMate,
}

#[derive(Clone, Debug)]
pub struct Genes {
    pub movement_speed: f32,
    pub vision_range: f32,
    pub vision_angle: f32,
    pub color: Color,
    pub width: f32,
    pub height: f32,
    pub reproduction_urge: f32,
    pub birth_energy_loss: f32,
    pub mature_age: i32,
    pub metabolism: f32,

    pub hunger_weight: f32,
    pub reproduction_weight: f32,
    pub energy_weight: f32,
    pub random_weight: f32,
}

#[derive(Component, Clone, Debug)]
pub struct Cell {
    pub health: f32,
    pub hunger: f32,
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
    pub mature: bool,
    pub is_moving: bool,
    pub offsprings_count: i32,
    pub reproduction_cooldown: f32,
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

        let genes = Genes {
            movement_speed: seeded_rng.gen_range(15.0..100.0),
            vision_range: seeded_rng.gen_range(100.0..400.0),
            color,
            vision_angle: seeded_rng.gen_range(10.0..180.0),
            width,
            height,
            reproduction_urge: seeded_rng.gen_range(0.0..100.0),
            birth_energy_loss: seeded_rng.gen_range(1.0..70.0),
            mature_age: seeded_rng.gen_range(5..18),
            metabolism: seeded_rng.gen_range(0.1..1.0),
            hunger_weight: seeded_rng.gen_range(1.1..1.2),
            reproduction_weight: seeded_rng.gen_range(1.1..1.2),
            energy_weight: seeded_rng.gen_range(1.1..1.2),
            random_weight: seeded_rng.gen_range(0.5..1.5),
        };

        return (
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(color))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            crate::common::Collider,
            Cell {
                fitness: 0.0,
                brain: NeuralNetwork::new(9, 10, 4),
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                generation: 0,
                hunger: 0.0,
                health: 100.0,
                energy: 100.0,
                age: 0,
                target_location: None,
                rotation: 0.0,
                action: Action::Chilling,
                genes,
                mature: false,
                is_moving: false,
                offsprings_count: 0,
                reproduction_cooldown: 0.0,
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
        gizmos.circle_2d(
            Vec2::new(self.pos_x, self.pos_y),
            self.genes.vision_range,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        let forward = Vec2::new(0.0, 1.0).rotate(Vec2::from_angle(self.rotation));
        let half_angle = self.genes.vision_angle.to_radians() / 2.0;
        let left = forward.rotate(Vec2::from_angle(-half_angle));
        let right = forward.rotate(Vec2::from_angle(half_angle));

        gizmos.line_2d(
            Vec2::new(self.pos_x, self.pos_y),
            Vec2::new(self.pos_x, self.pos_y) + left * self.genes.vision_range,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );
        gizmos.line_2d(
            Vec2::new(self.pos_x, self.pos_y),
            Vec2::new(self.pos_x, self.pos_y) + right * self.genes.vision_range,
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

        // match self.action {
        //     Action::Chilling => gizmos.circle_2d(Vec2::new(self.pos_x, self.pos_y), 10.0, BLUE),
        //     Action::GoingForFood => gizmos.circle_2d(Vec2::new(self.pos_x, self.pos_y), 10.0, RED),
        //     Action::MovingAround => {
        //         gizmos.circle_2d(Vec2::new(self.pos_x, self.pos_y), 10.0, YELLOW)
        //     }
        //     Action::FindMate => gizmos.circle_2d(Vec2::new(self.pos_x, self.pos_y), 10.0, PURPLE),
        // };
    }

    pub fn movement(&mut self, transform: &mut Transform, game_config: &GameConfig, time: f32) {
        if let Some(target) = self.target_location {
            let mut movement_speed = self.genes.movement_speed;
            // if self.energy < 30.0 {
            //     movement_speed *= 0.5;
            // }
            // if self.age < 80 {
            //     movement_speed *= 0.8;
            // }
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
            self.rotation = rotation.to_euler(EulerRot::XYZ).2;
            self.is_moving = true;
        } else {
            self.is_moving = false;
        }
    }

    pub fn process_metabolism(&mut self, time: f32) {
        self.hunger = (self.hunger + self.genes.metabolism * time).clamp(0.0, 100.0);

        if self.hunger == 100.0 {
            self.health = (self.health - (0.05 * time)).clamp(0.0, 100.0);
        }

        self.energy = (self.energy - self.genes.metabolism * time).clamp(0.0, 100.0);
    }

    pub fn process_brain(
        &mut self,
        rng: &mut ChaCha12Rng,
        all_cells: &Vec<Cell>,
        all_foods: &Vec<food::Food>,
        game_config: &common::GameConfig,
        gizmos: &mut Gizmos,
    ) {
        let hunger_score = (self.hunger / 100.0) * self.genes.hunger_weight;
        let reproduction_score = match self.genes.reproduction_urge {
            urge if urge > 75.0 => {
                if self.offsprings_count == 0 || self.offsprings_count == 1 {
                    (urge / 100.0) * self.genes.reproduction_weight * 1.5
                } else {
                    (urge / 100.0) * self.genes.reproduction_weight
                }
            }
            urge if urge > 40.0 => {
                if self.offsprings_count == 0 {
                    (urge / 100.0) * self.genes.reproduction_weight
                } else if self.offsprings_count == 1 {
                    (urge / 100.0) * self.genes.reproduction_weight * 0.8
                } else {
                    (urge / 100.0) * self.genes.reproduction_weight * 0.5
                }
            }
            _ => {
                if self.offsprings_count == 0 {
                    (self.genes.reproduction_urge / 100.0) * self.genes.reproduction_weight * 0.5
                } else {
                    (self.genes.reproduction_urge / 100.0) * self.genes.reproduction_weight * 0.2
                }
            }
        };

        let energy_score = (self.energy / 100.0) * self.genes.energy_weight;
        let low_energy = if self.energy < 30.0 { 1.0 } else { 0.0 };
        let high_hunger = if self.hunger > 70.0 { 1.0 } else { 0.0 };
        let is_healthy = if self.health > 70.0 { 1.0 } else { 0.0 };
        let random_score = self.genes.random_weight * rng.gen_range(0.0..1.0);

        let inputs = ndarray::array![
            hunger_score,
            self.health / 100.0,
            energy_score,
            low_energy,
            high_hunger,
            is_healthy,
            reproduction_score,
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
            0 => Action::Chilling,
            1 => Action::MovingAround,
            2 => Action::GoingForFood,
            3 => {
                if self.mature {
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
        &mut self,
        seeded_rng: &mut ChaCha12Rng,
        game_config: &GameConfig,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        parent2: &Cell,
    ) -> (
        bevy::prelude::Mesh2d,
        bevy::prelude::MeshMaterial2d<ColorMaterial>,
        bevy::prelude::Transform,
        crate::common::Collider,
        Cell,
    ) {
        self.reproduction_cooldown = 50.0;
        self.action = Action::Chilling;
        self.energy -= self.genes.birth_energy_loss;
        self.offsprings_count += 1;
        // parent2.reproduction_cooldown = 50.0;
        // parent2.action = Action::Chilling;
        // parent2.energy -= parent2.genes.birth_energy_loss;
        // parent2.offsprings_count += 1;
        let mut offspring_network = NeuralNetwork::crossover(&self.brain, &parent2.brain);
        Cell::mutate(&mut offspring_network, game_config);

        let width = blend(
            self.genes.width,
            parent2.genes.width,
            seeded_rng,
            game_config.mutation_rate,
        );
        let height = blend(
            self.genes.height,
            parent2.genes.height,
            seeded_rng,
            game_config.mutation_rate,
        );
        let movement_speed = blend(
            self.genes.movement_speed,
            parent2.genes.movement_speed,
            seeded_rng,
            game_config.mutation_rate,
        );
        let vision_range = blend(
            self.genes.vision_range,
            parent2.genes.vision_range,
            seeded_rng,
            game_config.mutation_rate,
        );
        let vision_angle = blend(
            self.genes.vision_angle,
            parent2.genes.vision_angle,
            seeded_rng,
            game_config.mutation_rate,
        );
        let reproduction_urge = blend(
            self.genes.reproduction_urge,
            parent2.genes.reproduction_urge,
            seeded_rng,
            game_config.mutation_rate,
        );
        let birth_energy_loss = blend(
            self.genes.birth_energy_loss,
            parent2.genes.birth_energy_loss,
            seeded_rng,
            game_config.mutation_rate,
        );
        let mature_age = blend(
            self.genes.mature_age as f32,
            parent2.genes.mature_age as f32,
            seeded_rng,
            game_config.mutation_rate,
        ) as i32;
        let metabolism = blend(
            self.genes.metabolism,
            parent2.genes.metabolism,
            seeded_rng,
            game_config.mutation_rate,
        );
        let color = blend_colors(
            self.genes.color.to_linear(),
            parent2.genes.color.to_linear(),
            seeded_rng,
        );
        let hunger_weight = blend(
            self.genes.hunger_weight,
            parent2.genes.hunger_weight,
            seeded_rng,
            game_config.mutation_rate,
        )
        .clamp(0.0, 2.0);
        let reproduction_weight = blend(
            self.genes.reproduction_weight,
            parent2.genes.reproduction_weight,
            seeded_rng,
            game_config.mutation_rate,
        )
        .clamp(0.0, 2.0);
        let energy_weight = blend(
            self.genes.energy_weight,
            parent2.genes.energy_weight,
            seeded_rng,
            game_config.mutation_rate,
        )
        .clamp(0.0, 2.0);
        let random_weight = blend(
            self.genes.random_weight,
            parent2.genes.random_weight,
            seeded_rng,
            game_config.mutation_rate,
        )
        .clamp(0.0, 2.0);
        let mut highest_generation = self.generation;

        if parent2.generation > highest_generation {
            highest_generation = parent2.generation;
        }

        let x = self.pos_x;
        let y = self.pos_y;

        let genes = Genes {
            movement_speed,
            vision_range,
            color,
            vision_angle,
            width,
            height,
            reproduction_urge,
            birth_energy_loss,
            mature_age,
            metabolism,
            hunger_weight,
            reproduction_weight,
            energy_weight,
            random_weight,
        };

        return (
            Mesh2d(meshes.add(Rectangle::new(width, height))),
            MeshMaterial2d(materials.add(Color::from(color))),
            Transform::default().with_translation(Vec3::new(x, y, 0.0)),
            crate::common::Collider,
            Cell {
                fitness: 0.0,
                brain: offspring_network,
                id: Uuid::new_v4(),
                pos_x: x,
                pos_y: y,
                hunger: 0.0,
                health: 100.0,
                energy: 100.0,
                age: 0,
                target_location: None,
                rotation: 0.0,
                action: Action::Chilling,
                generation: highest_generation + 1,
                genes,
                mature: false,
                is_moving: false,
                offsprings_count: 0,
                reproduction_cooldown: 0.0,
            },
        );
    }

    pub fn rest(&mut self) {
        self.target_location = None;
        self.energy = (self.energy + if self.energy < 30.0 { 1.5 } else { 0.5 }).clamp(0.0, 100.0);
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
        self.hunger = (self.hunger - 15.0).clamp(0.0, 100.0);
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
