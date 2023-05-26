use anyhow::anyhow;
use axum::{
    extract::{Json, Path, State},
    http::header::{HeaderMap, HeaderValue},
};
use http::{
    header::{CONTENT_TYPE, REFERER},
    Response, StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};

use svc_authn::AccountId;
use svc_utils::extractors::AccountIdExtractor;

use super::{s3_object, valid_referer, wrap_error};
use crate::app::{authz::AuthzObject, context::AppContext, util::S3SignedRequestBuilder};

#[derive(Debug, Deserialize)]
pub struct SignPayload {
    set: String,
    object: String,
    method: String,
    headers: BTreeMap<String, String>,
}

pub async fn backend_sign(
    State(ctx): State<Arc<AppContext>>,
    AccountIdExtractor(sub): AccountIdExtractor,
    Path(back): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<SignPayload>,
) -> Response<String> {
    unimplemented!();
}
