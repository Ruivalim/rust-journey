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
    let mut oldest = 0;

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

        if cell.age > oldest {
            oldest = cell.age;
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
        ui.label(format!("Cells dead: {}", game_options.dead_cells));
        ui.label(format!("Current Day: {}", game_options.current_day));
        ui.label(format!("Highest Generation: {}", highest_generation));
        ui.label(format!("Oldest: {}", oldest));
        ui.checkbox(&mut game_options.show_fittest, "Show Fittest");
        ui.checkbox(&mut game_options.debug_logs, "Debug Logs");
        ui.checkbox(&mut game_options.paused, "Pause");
        ui.add(egui::Slider::new(&mut game_options.foods_per_day, 0..=10).text("Foods Per Day"));
        ui.add(egui::Slider::new(&mut game_options.mutation_rate, 0.1..=1.0).text("Mutation Rate"));
        ui.add(egui::Slider::new(&mut game_options.day_speed, 0.1..=100.0).text("Day Speed"));
        ui.add(egui::Slider::new(&mut game_options.map_height, 100.0..=20000.0).text("Map Height"));
        ui.add(egui::Slider::new(&mut game_options.map_width, 100.0..=20000.0).text("Map Width"));
    });

    egui::Window::new("Cell Viewer").show(contexts.ctx_mut(), |ui| {
        if let Some(cell) = &mut selected_cell.0 {
            ui.label(format!("Cell ID: {}", cell.id.to_string()));
            ui.collapsing("Genes", |ui| {
                ui.label("Genes");
                ui.label(format!("Speed: {}", cell.genes.movement_speed));
                ui.label(format!("Vision: {:?}", cell.genes.vision_range));
                ui.label(format!("Vision Angle: {:?}", cell.genes.vision_angle));
                ui.label(format!("Metabolism: {:?}", cell.genes.metabolism));
                ui.label(format!("Energy Weight: {:?}", cell.genes.energy_weight));
                ui.label(format!("Random Weight: {:?}", cell.genes.random_weight));
            });
            ui.collapsing("General Infos", |ui| {
                ui.label(format!("Energy: {}", cell.energy));
                ui.label(format!("Health: {}", cell.health));
                ui.label(format!("Target: {:?}", cell.target_location));
                ui.label(format!("Action: {:?}", cell.action));
                ui.label(format!("Fitness: {:?}", cell.fitness));
                ui.label(format!("Age: {:?}", cell.age));
                ui.label(format!("Generation: {:?}", cell.generation));
                ui.label(format!("Offsprings: {:?}", cell.offsprings_count));
                ui.label(format!("X: {:?}", cell.pos_x));
                ui.label(format!("Y: {:?}", cell.pos_y));
            });
        }
        ui.separator();
        if ui.button("Clear Selected").clicked() {
            selected_cell.0 = None;
        }
    });

    // egui::Window::new("Brain Cell Viewer").show(contexts.ctx_mut(), |ui| {
    //     if let Some(cell) = &mut selected_cell.0 {
    //         let brain = &cell.brain;

    //         // Configuration for visualizer
    //         let neuron_radius = 10.0;
    //         let layer_spacing = 150.0;
    //         let neuron_spacing = 30.0;

    //         let mut cursor_x = ui.next_widget_position().x + 50.0; // Starting X position for the visualization
    //         let mut cursor_y = ui.next_widget_position().y + 50.0; // Starting Y position

    //         let input_neurons = brain.weights_input_hidden.len(); // Number of input neurons
    //         let hidden_neurons = brain.weights_hidden_output.len(); // Number of hidden neurons
    //         let output_neurons = brain.biases_output.len(); // Number of output neurons

    //         // Draw Input Layer
    //         for i in 0..input_neurons {
    //             let y = cursor_y + i as f32 * neuron_spacing;
    //             ui.painter().circle_filled(
    //                 egui::Pos2::new(cursor_x, y),
    //                 neuron_radius,
    //                 egui::Color32::BLUE,
    //             );
    //         }

    //         cursor_x += layer_spacing;

    //         for j in 0..hidden_neurons {
    //             let y_hidden = cursor_y + j as f32 * neuron_spacing;
    //             ui.painter().circle_filled(
    //                 egui::Pos2::new(cursor_x, y_hidden),
    //                 neuron_radius,
    //                 egui::Color32::GREEN,
    //             );

    //             // Connect Input to Hidden with Lines (Weights)
    //             // for i in 0..input_neurons {
    //             //     let y_input = cursor_y + i as f32 * neuron_spacing;
    //             //     let weight = brain.weights_input_hidden[[i, j]];
    //             //     let line_color = if weight > 0.0 {
    //             //         egui::Color32::RED
    //             //     } else {
    //             //         egui::Color32::BLUE
    //             //     };

    //             //     ui.painter().line_segment(
    //             //         [
    //             //             egui::Pos2::new(cursor_x - layer_spacing, y_input),
    //             //             egui::Pos2::new(cursor_x, y_hidden),
    //             //         ],
    //             //         egui::Stroke::new(weight.abs() * 2.0, line_color),
    //             //     );
    //             // }
    //         }

    //         // Move cursor to the next layer (Output Layer)
    //         cursor_x += layer_spacing;

    //         // Draw Output Layer
    //         for k in 0..output_neurons {
    //             let y_output = cursor_y + k as f32 * neuron_spacing;
    //             ui.painter().circle_filled(
    //                 egui::Pos2::new(cursor_x, y_output),
    //                 neuron_radius,
    //                 egui::Color32::RED,
    //             );

    //             // Connect Hidden to Output with Lines (Weights)
    //             // for j in 0..hidden_neurons {
    //             //     let y_hidden = cursor_y + j as f32 * neuron_spacing;
    //             //     let weight = brain.weights_hidden_output[[j, k]];
    //             //     let line_color = if weight > 0.0 {
    //             //         egui::Color32::RED
    //             //     } else {
    //             //         egui::Color32::BLUE
    //             //     };

    //             //     ui.painter().line_segment(
    //             //         [
    //             //             egui::Pos2::new(cursor_x - layer_spacing, y_hidden),
    //             //             egui::Pos2::new(cursor_x, y_output),
    //             //         ],
    //             //         egui::Stroke::new(weight.abs() * 2.0, line_color),
    //             //     );
    //             // }
    //         }

    //         ui.set_height(cursor_x + 50.0);
    //         ui.set_width(cursor_y + 50.0);
    //     }
    // });
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
