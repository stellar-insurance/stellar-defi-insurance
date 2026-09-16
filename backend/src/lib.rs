pub mod error;
pub mod models;
pub mod routes;
pub mod services;
pub mod state;

pub fn app_router(state: state::AppState) -> axum::Router {
    routes::build_router(state)
}
