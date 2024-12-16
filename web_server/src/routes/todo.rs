use axum::{
    extract::{Path, State},
    response::Json,
    routing::{delete, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::{Arc, MutexGuard};
use uuid::Uuid;

use crate::structs::{AddItemBody, AppState, Todo};

async fn add_todo(Json(payload): Json<AddItemBody>, state: Arc<AppState>) -> Json<Value> {
    let mut todo = state.todo.lock().unwrap();

    let new_item = Todo {
        id: Uuid::new_v4().to_string(),
        title: payload.title,
    };
    todo.push(new_item.clone());

    Json(json!(new_item.clone()))
}

async fn list_items(State(state): State<Arc<AppState>>) -> Json<Value> {
    let todo: MutexGuard<'_, Vec<Todo>> = state.todo.lock().unwrap();
    Json(json!(todo.clone()))
}

async fn remove_todo(Path(item_id): Path<String>, state: Arc<AppState>) -> Json<Value> {
    let mut todo = state.todo.lock().unwrap();
    let index = match todo.iter().position(|x| x.id.eq(&item_id)) {
        Some(index) => index,
        None => return Json(json!({"error": "Item not found"})),
    };
    let removed_item = todo.remove(index);

    Json(json!(removed_item))
}

pub fn todo_routes(state: &Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            post({
                let shared_state = Arc::clone(&state);
                move |body| add_todo(body, shared_state)
            })
            .get(list_items),
        )
        .route(
            "/:id",
            delete({
                let shared_state = Arc::clone(&state);
                move |path| remove_todo(path, shared_state)
            }),
        )
        .with_state(state.clone())
}
