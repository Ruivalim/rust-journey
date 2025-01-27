use bevy::{input::common_conditions::input_pressed, prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::common;

pub fn ui_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            ui_system,
            mouse_coordinates_system,
            get_cell_info.run_if(input_pressed(MouseButton::Left)),
            update_cell_info,
        ),
    );
}

pub fn mouse_coordinates_system(
    mut mouse_coordinates: ResMut<common::MouseCoordinates>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<common::MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        mouse_coordinates.0 = world_position;
    }
}

pub fn ui_system(
    mut contexts: EguiContexts,
    selected_cell: Res<common::CellSelected>,
    query_cells: Query<&common::Cell>,
    query_foods: Query<&common::Food>,
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
            ui.label(format!("Health: {}", cell.health));
            ui.label(format!("Pos X: {}", cell.pos_x));
            ui.label(format!("Pox Y: {}", cell.pos_y));
            ui.label(format!("Speed: {}", cell.movement_speed));
            ui.label(format!("Parent 1: {:?}", cell.parent_1));
            ui.label(format!("Parent 2: {:?}", cell.parent_2));
            ui.label(format!("Action: {:?}", cell.action));
            ui.label(format!("Timer: {:?}", cell.action_timer));
            ui.label(format!("Vision: {:?}", cell.vision_range));
        } else {
            ui.label("No cell selected");
        }
    });
}

pub fn get_cell_info(
    mut selected_cell: ResMut<common::CellSelected>,
    queries: Query<(&common::Cell), With<common::Cell>>,
    mouse_coordinates: ResMut<common::MouseCoordinates>,
) {
    let m_x = mouse_coordinates.0.x;
    let m_y = mouse_coordinates.0.y;

    let cells_on_position = queries.iter().filter(|cell| {
        let s_width = cell.width / 2.0;
        let s_height = cell.height / 2.0;
        let x = cell.pos_x;
        let y = cell.pos_y;
        let x_min = x - s_width;
        let x_max = x + s_width;
        let y_min = y - s_height;
        let y_max = y + s_height;

        m_x >= x_min && m_x <= x_max && m_y >= y_min && m_y <= y_max
    });

    let cells_on_position = cells_on_position.collect::<Vec<&common::Cell>>();
    let count = cells_on_position.len();

    if count == 0 as usize {
        selected_cell.0 = None;
        return;
    }

    selected_cell.0 = Some(cells_on_position[0].clone());
}

pub fn update_cell_info(
    mut selected_cell: ResMut<common::CellSelected>,
    queries: Query<(&common::Cell), With<common::Cell>>,
) {
    if let Some(cell_sel) = &selected_cell.0 {
        let cell_selected = queries.iter().filter(|cell| cell.id.eq(&cell_sel.id));
        let cell_selected = cell_selected.collect::<Vec<&common::Cell>>();
        let count = cell_selected.len();

        if count == 0 as usize {
            selected_cell.0 = None;
            return;
        }

        selected_cell.0 = Some(cell_selected[0].clone());
    }
}
