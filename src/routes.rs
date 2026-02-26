use axum::routing::get;
use axum::Router;

use crate::handlers::{create_plant, delete_plant, get_plant, list_plants, update_plant};
use crate::store::PlantStore;

pub fn app(store: PlantStore) -> Router {
    Router::new()
        .route("/plants", get(list_plants).post(create_plant))
        .route(
            "/plants/:id",
            get(get_plant).put(update_plant).delete(delete_plant),
        )
        .with_state(store)
}
