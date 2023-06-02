use axum::{
    body::Body,
    routing::{get, post},
    Extension, Router,
    Extension, Router,
};
use http::{
    header::{self, HeaderName},
    Method, Response,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::error;

use super::{config::AppConfig, context::AppContext, endpoints};

pub fn build_router(context: Arc<AppContext>) -> Router {
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
            HeaderName::from_static("x-request-type"),
            HeaderName::from_static("x-agent-label"),
        ])
        .max_age(std::time::Duration::from_secs(3600))
        .allow_origin(Any);

    let routes = Router::new().nest(
        "/api/v2",
        Router::new()
            .route(
                "/backends/:back/sets/:set/objects/:object",
                get(endpoints::backend_read),
            )
            .route("/backends/:back/sign", post(endpoints::backend_sign))
            .layer(cors)
            .layer(Extension(Arc::new(authn)))
            .layer(Extension(Arc::new(context.application_id.clone())))
            .layer(Extension(maxmind))
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
    let ctx = Arc::new(AppContext::build(config.clone()));

    let reader = Arc::new(
        maxminddb::Reader::open_readfile("maximind.mmdb").expect("can't load maxminddb"),
    );

    if let Err(e) = axum::Server::bind(&config.http.listener_address)
        .serve(build_router(ctx, config.authn.clone(), reader).into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        error!("Failed to await http server completion, err = {:?}", e);
    }
}
