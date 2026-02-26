use axum::Router;

pub mod aggregate;
pub mod history;

pub fn get_routes(state: crate::AppState) -> Router {
    Router::new()
        .route(
            "/aggregate",
            axum::routing::post(aggregate::handler::handler),
        )
        .route("/history", axum::routing::get(history::handler::handler))
        .with_state(state)
}
