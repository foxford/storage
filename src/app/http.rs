use std::sync::Arc;

use axum::{
    body::Body,
    routing::{get, post},
    Extension, Router,
};

use http::{
    header::{self, HeaderName},
    Method, Response,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::error;

use super::{config::AppConfig, context::AppContext, endpoints};

pub fn build_router(context: Arc<AppContext>, authn: svc_authn::jose::ConfigMap) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            header::AUTHORIZATION,
            header::CACHE_CONTROL,
            header::CONTENT_LENGTH,
            header::CONTENT_TYPE,
            header::IF_MATCH,
            header::IF_MODIFIED_SINCE,
            header::IF_NONE_MATCH,
            header::IF_UNMODIFIED_SINCE,
            header::RANGE,
            header::HeaderName::from_static("x-request-type"),
            HeaderName::from_static("x-agent-label"),
        ])
        .max_age(std::time::Duration::from_secs(3600))
        .allow_origin(Any);

    let routes = Router::new().nest(
        "/api/v2",
        Router::new()
            .route("/sets/:set/objects/:object", get(endpoints::read))
            .route(
                "/backends/:back/sets/:set/objects/:object",
                get(endpoints::backend_read),
            )
            .route("/sign", post(endpoints::sign))
            .route("/backends/:back/sign", post(endpoints::backend_sign))
            .layer(cors)
            .layer(Extension(Arc::new(authn)))
            .with_state(context),
    );

    let pingz_router = Router::new().route(
        "/healthz",
        get(|| async { Response::builder().body(Body::from("pong")).unwrap() }),
    );

    let routes = routes.merge(pingz_router);

    routes.layer(svc_utils::middleware::LogLayer::new())
}

pub async fn run(config: AppConfig) {
    let context = AppContext::build(config.clone());
    let ctx = Arc::new(context);

    if let Err(e) = axum::Server::bind(&config.http.listener_address)
        .serve(build_router(ctx, config.authn.clone()).into_make_service())
        .await
    {
        error!("Failed to await http server completion, err = {:?}", e);
    }
}