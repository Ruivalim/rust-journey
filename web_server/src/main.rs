use axum::{
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Serialize, Clone)]
struct Todo {
    id: String,
    title: String,
}

#[derive(serde::Deserialize)]
struct AddItemBody {
    title: String,
}

struct AppState {
    todo: Mutex<Vec<Todo>>,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        todo: Mutex::new(vec![]),
    });

    let router = Router::new()
        .route("/", get(home))
        .route("/about", get(about))
        .route(
            "/todo",
            post({
                let shared_state = Arc::clone(&shared_state);
                move |body| add_todo(body, shared_state)
            })
            .get(list_items),
        )
        .route(
            "/todo/:id",
            delete({
                let shared_state = Arc::clone(&shared_state);
                move |path| remove_todo(path, shared_state)
            }),
        )
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn home() -> &'static str {
    "Home Page"
}

async fn about() -> &'static str {
    "About Page"
}

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
    let todo: std::sync::MutexGuard<'_, Vec<Todo>> = state.todo.lock().unwrap();

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
