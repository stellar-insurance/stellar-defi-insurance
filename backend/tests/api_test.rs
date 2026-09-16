use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn test_app() -> axum::Router {
    let config = stellar_defi_insurance_backend::state::Config {
        bind_addr: "0.0.0.0:0".parse().unwrap(),
        contract_id: String::new(),
        soroban_rpc_url: "http://localhost".into(),
        horizon_url: "http://localhost".into(),
        network_passphrase: "Test SDF Network ; September 2015".into(),
        network_name: "TESTNET".into(),
    };
    let state = stellar_defi_insurance_backend::state::AppState::new(config);
    state.store.seed_demo_data();
    stellar_defi_insurance_backend::app_router(state)
}

async fn body_str(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn health_ok() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"status\":\"ok\""));
}

#[tokio::test]
async fn status_has_pool_info() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/status").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_str(res.into_body()).await;
    assert!(body.contains("\"pool_balance\""));
    assert!(body.contains("\"policy_count\":4"));
}

#[tokio::test]
async fn list_policies() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/policies").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"coverage_amount\""));
}

#[tokio::test]
async fn get_policy_by_id() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/policies/0").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"id\":0"));
}

#[tokio::test]
async fn get_nonexistent_policy_404() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/policies/999").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_claims() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/claims").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body_str(res.into_body()).await;
    assert!(body.contains("\"claim_amount\""));
}

#[tokio::test]
async fn pool_info() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/pool").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"pool_balance\""));
}

#[tokio::test]
async fn policy_count() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/policies/count").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"count\":4"));
}

#[tokio::test]
async fn check_assessor() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/assessors/GA2C5RFPE6GCKMY3US5GAB29QZYE7DYXTZPGHLDJ5YBJO7VOZKAUBP2X").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"is_assessor\":true"));
}

#[tokio::test]
async fn claim_count() {
    let app = test_app();
    let res = app.oneshot(Request::builder().uri("/api/v1/claims/count").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(body_str(res.into_body()).await.contains("\"count\":1"));
}
