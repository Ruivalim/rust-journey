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
    selected_cell: Res<common::CellSelected>,
    query_cells: Query<&cell::Cell>,
    query_foods: Query<&food::Food>,
    mut game_options: ResMut<common::GameConfig>,
) {
    egui::Window::new("Game Options").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Foods: {}", query_foods.iter().count()));
        ui.label(format!("Cells alive: {}", query_cells.iter().count()));
        ui.add(
            egui::Slider::new(&mut game_options.movement_cost, 0.001..=1.0).text("Movement Cost"),
        );
        ui.add(
            egui::Slider::new(&mut game_options.food_spawn_rate, 0.0..=10.0)
                .text("Food spawn Rate"),
        );
    });

    egui::Window::new("Cell Viewer").show(contexts.ctx_mut(), |ui| {
        ui.separator();
        if let Some(cell) = &selected_cell.0 {
            ui.label(format!("ID: {}", cell.id.to_string()));
            ui.label(format!("Width: {}", cell.width));
            ui.label(format!("Height: {}", cell.height));
            ui.label(format!("Energy: {}", cell.energy));
            ui.label(format!("Health: {}", cell.health));
            ui.label(format!("Pos X: {}", cell.pos_x));
            ui.label(format!("Pox Y: {}", cell.pos_y));
            ui.label(format!("Speed: {}", cell.movement_speed));
            ui.label(format!("Vision: {:?}", cell.vision_range));
            ui.label(format!("Action: {:?}", cell.action));
        } else {
            ui.label("No cell selected");
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
