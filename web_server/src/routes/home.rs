use axum::{routing::get, Router};
use std::sync::Arc;

use crate::structs::AppState;

async fn show_home(state: Arc<AppState>) -> String {
    let todo = state.todo.lock().unwrap();
    let count = todo.len();

    format!("{} items in the todo list\n", count)
}

pub fn home_routes(state: &Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            get({
                let shared_state = Arc::clone(&state);
                move || show_home(shared_state)
            }),
        )
        .with_state(state.clone())
}
