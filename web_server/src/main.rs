mod routes;
mod structs;

use axum::{routing::get, Router};
use routes::{home::home_routes, todo::todo_routes};
use std::sync::{Arc, Mutex};
use structs::AppState;

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        todo: Mutex::new(vec![]),
    });

    let router = Router::new()
        .nest("/", home_routes(&shared_state))
        .route("/about", get(about))
        .nest("/todo", todo_routes(&shared_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn about() -> &'static str {
    "About Page"
}
