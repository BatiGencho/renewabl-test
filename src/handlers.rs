use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

use crate::models::{CreatePlantRequest, UpdatePlantRequest};
use crate::store::PlantStore;

pub async fn list_plants(State(store): State<PlantStore>) -> impl IntoResponse {
    match store.list() {
        Ok(plants) => (StatusCode::OK, Json(plants)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_plant(
    State(store): State<PlantStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match store.get(id) {
        Ok(plant) => (StatusCode::OK, Json(plant)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn create_plant(
    State(store): State<PlantStore>,
    Json(req): Json<CreatePlantRequest>,
) -> impl IntoResponse {
    match store.create(req) {
        Ok(plant) => (StatusCode::CREATED, Json(plant)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn update_plant(
    State(store): State<PlantStore>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePlantRequest>,
) -> impl IntoResponse {
    match store.update(id, req) {
        Ok(plant) => (StatusCode::OK, Json(plant)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn delete_plant(
    State(store): State<PlantStore>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match store.delete(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}
