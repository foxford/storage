use axum::{
    extract::{Path, State},
    http::header::{HeaderMap, HeaderValue},
};
use http::{header::REFERER, Response, StatusCode};
use std::sync::Arc;

use svc_authn::AccountId;

use super::{s3_object, valid_referer, wrap_error};
use crate::app::{access_token::AccessTokenExtractor, authz::AuthzObject, context::AppContext};

pub async fn read(
    State(ctx): State<Arc<AppContext>>,
    AccessTokenExtractor(sub): AccessTokenExtractor,
    Path((set, object)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response<String> {
    let back = String::from(crate::app::util::S3_DEFAULT_CLIENT);
    read_ns(ctx, back, set, object, sub, headers.get(REFERER)).await
}

pub async fn backend_read(
    State(ctx): State<Arc<AppContext>>,
    AccessTokenExtractor(sub): AccessTokenExtractor,
    Path((back, set, object)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Response<String> {
    read_ns(ctx, back, set, object, sub, headers.get(REFERER)).await
}

async fn read_ns(
    ctx: Arc<AppContext>,
    back: String,
    set: String,
    object: String,
    sub: AccountId,
    referer: Option<&HeaderValue>,
) -> Response<String> {
    let zobj = AuthzObject::new(&["sets", &set]);
    let zact = "read";
    let s3 = match ctx.s3.get(&back) {
        Some(val) => val.clone(),
        None => {
            return wrap_error(
                StatusCode::NOT_FOUND,
                format!(
                    "Error reading an object by set: Backend '{}' is not found",
                    &back
                ),
            )
        }
    };

    match ctx.aud_estm.parse_set(&set) {
        Ok(set_s) => {
            if let Err(err) = valid_referer(&ctx, &set_s.bucket().to_string(), referer) {
                return err;
            }

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
                    format!("Error reading an object by set: {}", err),
                ),
                Ok(_) => {
                    let bucket = set_s.bucket().to_string();
                    let object = s3_object(set_s.label(), &object);

                    match s3.presigned_url("GET", &bucket, &object) {
                        Ok(uri) => redirect(uri),
                        Err(err) => wrap_error(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            format!("Error reading an object by set: {}", err),
                        ),
                    }
                }
            }
        }
        Err(err) => wrap_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error reading an object by set: {}", err),
        ),
    }
}

fn redirect(uri: String) -> Response<String> {
    Response::builder()
        .header("location", uri)
        .header("Timing-Allow-Origin", "*")
        .status(StatusCode::SEE_OTHER)
        .body(String::default())
        .unwrap()
}
