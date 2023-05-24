use axum::{
    extract::{Path, State},
    Json,
};
use http::Response;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;

use svc_utils::extractors::AccountIdExtractor;

use crate::app::context::AppContext;

#[derive(Debug, Deserialize)]
pub struct SignPayload {
    set: String,
    object: String,
    method: String,
    headers: BTreeMap<String, String>,
}

pub async fn sign(
    State(ctx): State<Arc<AppContext>>,
    AccountIdExtractor(sub): AccountIdExtractor,
    Json(payload): Json<SignPayload>,
) -> Response<String> {
    unimplemented!();
}

pub async fn backend_sign(
    State(ctx): State<Arc<AppContext>>,
    AccountIdExtractor(sub): AccountIdExtractor,
    Path(back): Path<String>,
    Json(payload): Json<SignPayload>,
) -> Response<String> {
    unimplemented!();
}
