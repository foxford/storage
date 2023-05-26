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
    sign_ns(ctx, back, payload, sub, headers.get(REFERER)).await
}

async fn sign_ns(
    ctx: Arc<AppContext>,
    back: String,
    body: SignPayload,
    sub: AccountId,
    referer: Option<&HeaderValue>,
) -> Response<String> {
    if let Ok(set_s) = ctx.aud_estm.parse_set(&body.set) {
        if let Err(err) = valid_referer(&ctx, &set_s.bucket().to_string(), referer) {
            return err;
        }
    }

    let zobj = AuthzObject::new(&["sets", &body.set]);
    let zact = match parse_action(&body.method) {
        Ok(val) => val,
        Err(err) => {
            return wrap_error(
                StatusCode::FORBIDDEN,
                format!("Error signing a request: {}", err),
            )
        }
    };

    let s3 = match ctx.s3.get(&back) {
        Some(val) => val.clone(),
        None => {
            return wrap_error(
                StatusCode::NOT_FOUND,
                format!("Error signing a request: Backend '{}' is not found", &back),
            )
        }
    };

    match ctx.aud_estm.parse_set(&body.set) {
        Ok(set_s) => {
            match ctx
                .authz
                .authorize(
                    set_s.bucket().audience().to_string(),
                    sub,
                    Box::new(zobj),
                    zact.to_string(),
                )
                .await
            {
                Err(err) => wrap_error(
                    StatusCode::FORBIDDEN,
                    format!("Error signing a request: {}", err),
                ),
                Ok(_) => {
                    // URI builder
                    let mut builder = S3SignedRequestBuilder::new()
                        .method(&body.method)
                        .bucket(&set_s.bucket().to_string())
                        .object(&s3_object(set_s.label(), &body.object));
                    for (key, val) in body.headers {
                        builder = builder.add_header(&key, &val);
                    }
                    match builder.build(&s3) {
                        Ok(uri) => Response::builder()
                            .status(StatusCode::OK)
                            .header(CONTENT_TYPE, "application/json")
                            .body(
                                json!({
                                    "uri": uri,
                                })
                                .to_string(),
                            )
                            .unwrap(),
                        Err(err) => wrap_error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Error signing a request: {}", err),
                        ),
                    }
                }
            }
        }
        Err(err) => wrap_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error signing a request: {}", err),
        ),
    }
}

pub fn parse_action(method: &str) -> anyhow::Result<&str> {
    match method {
        "HEAD" => Ok("read"),
        "GET" => Ok("read"),
        "PUT" => Ok("update"),
        "DELETE" => Ok("delete"),
        _ => Err(anyhow!("invalid method = {}", method)),
    }
}
