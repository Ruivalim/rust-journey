use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Clone)]
pub struct Todo {
    pub id: String,
    pub title: String,
}

#[derive(Deserialize)]
pub struct AddItemBody {
    pub title: String,
}

pub struct AppState {
    pub todo: Mutex<Vec<Todo>>,
}
