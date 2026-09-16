use stellar_defi_insurance_backend::{app_router, state::AppState, state::Config};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,stellar_defi_insurance_backend=debug".into()))
        .with_target(true).init();

    let config = Config::from_env();
    tracing::info!(bind = %config.bind_addr, "Starting Stellar DeFi Insurance backend");

    let app_state = AppState::new(config.clone());
    if std::env::var("SKIP_SEED").unwrap_or_default() != "1" {
        app_state.store.seed_demo_data();
    }

    let app = app_router(app_state).layer(TraceLayer::new_for_http()).layer(CorsLayer::permissive());
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await.unwrap_or_else(|e| panic!("Bind failed: {e}"));
    tracing::info!("Listening on http://{}", config.bind_addr);
    axum::serve(listener, app).with_graceful_shutdown(async { tokio::signal::ctrl_c().await.ok(); }).await.expect("server error");
}
