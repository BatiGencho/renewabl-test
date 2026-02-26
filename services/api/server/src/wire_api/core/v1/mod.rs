use axum::Router;

pub(crate) mod energy;
pub(crate) mod errors;
pub(crate) mod types;

pub fn get_routes(state: crate::AppState) -> Router {
    Router::new().nest("/energy", energy::get_routes(state))
}
