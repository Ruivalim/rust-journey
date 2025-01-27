use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::cell;
use crate::common;
use crate::food;

pub fn ui_plugin(app: &mut App) {
    app.add_systems(Update, (ui_system, update_cell_info));
}

pub fn ui_system(
    mut contexts: EguiContexts,
    query_cells: Query<&cell::Cell>,
    query_foods: Query<&food::Food>,
    mut game_options: ResMut<common::GameConfig>,
    mut selected_cell: ResMut<common::CellSelected>,
) {
    let mut highest_generation = 0;
    let mut highest_offspring: Option<&cell::Cell> = None;

    for cell in query_cells.iter() {
        if cell.generation > highest_generation {
            highest_generation = cell.generation;
        }

        if let Some(fittest) = highest_offspring {
            if fittest.fitness < cell.fitness {
                highest_offspring = Some(cell);
            }
        } else {
            highest_offspring = Some(cell);
        }
    }

    if game_options.show_fittest {
        if let Some(fittest) = highest_offspring {
            selected_cell.0 = Some(fittest.clone());
        }
    }

    egui::Window::new("Game Options").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Foods: {}", query_foods.iter().count()));
        ui.label(format!("Cells alive: {}", query_cells.iter().count()));
        ui.label(format!("Current Day: {}", game_options.current_day));
        ui.label(format!("Highest Generation: {}", highest_generation));
        ui.checkbox(&mut game_options.show_fittest, "Show Fittest");
        ui.checkbox(&mut game_options.debug_logs, "Debug Logs");
        ui.add(egui::Slider::new(&mut game_options.foods_per_day, 0..=10).text("Foods Per Day"));
        ui.add(egui::Slider::new(&mut game_options.mutation_rate, 0.1..=1.0).text("Mutation Rate"));
        ui.add(egui::Slider::new(&mut game_options.day_speed, 1.0..=100.0).text("Day Speed"));
    });

    egui::Window::new("Cell Viewer").show(contexts.ctx_mut(), |ui| {
        if let Some(cell) = &mut selected_cell.0 {
            ui.label(format!("Cell ID: {}", cell.id.to_string()));
            ui.collapsing("Genes", |ui| {
                ui.label("Genes");
                ui.label(format!("Width: {}", cell.genes.width));
                ui.label(format!("Height: {}", cell.genes.height));
                ui.label(format!("Speed: {}", cell.genes.movement_speed));
                ui.label(format!("Vision: {:?}", cell.genes.vision_range));
                ui.label(format!("Vision Angle: {:?}", cell.genes.vision_angle));
                ui.label(format!("Metabolism: {:?}", cell.genes.metabolism));
                ui.label(format!("Mature Age: {:?}", cell.genes.mature_age));
                ui.label(format!(
                    "Birth Energy Loss: {:?}",
                    cell.genes.birth_energy_loss
                ));
                ui.label(format!(
                    "Reproduction Urge: {:?}",
                    cell.genes.reproduction_urge
                ));
                ui.label(format!("Hunger Weight: {:?}", cell.genes.hunger_weight));
                ui.label(format!("Energy Weight: {:?}", cell.genes.energy_weight));
                ui.label(format!(
                    "Reproduction Weight: {:?}",
                    cell.genes.reproduction_weight
                ));
                ui.label(format!("Random Weight: {:?}", cell.genes.random_weight));
            });
            ui.collapsing("General Infos", |ui| {
                ui.label(format!("Energy: {}", cell.energy));
                ui.label(format!("Health: {}", cell.health));
                ui.label(format!("Target: {:?}", cell.target_location));
                ui.label(format!("Action: {:?}", cell.action));
                ui.label(format!("Hunger: {:?}", cell.hunger));
                ui.label(format!("Fitness: {:?}", cell.fitness));
                ui.label(format!("Age: {:?}", cell.age));
                ui.label(format!("Generation: {:?}", cell.generation));
                ui.label(format!("Offsprings: {:?}", cell.offsprings_count));
            });
        }
        ui.separator();
        if ui.button("Clear Selected").clicked() {
            selected_cell.0 = None;
        }
    });
}

pub fn update_cell_info(
    mut selected_cell: ResMut<common::CellSelected>,
    queries: Query<&cell::Cell, With<cell::Cell>>,
) {
    if let Some(cell_sel) = &selected_cell.0 {
        let cell_selected = queries.iter().filter(|cell| cell.id.eq(&cell_sel.id));
        let cell_selected = cell_selected.collect::<Vec<&cell::Cell>>();
        let count = cell_selected.len();

        if count == 0 as usize {
            selected_cell.0 = None;
            return;
        }

        selected_cell.0 = Some(cell_selected[0].clone());
    }
}
