use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    http::header::{HeaderMap, HeaderValue},
};
use http::{header::REFERER, Response, StatusCode};
use std::sync::Arc;

use svc_authn::AccountId;
use svc_utils::extractors::AccountIdExtractor;
use tracing::error;

use crate::app::{authz::AuthzObject, context::AppContext};

pub async fn read(
    State(ctx): State<Arc<AppContext>>,
    AccountIdExtractor(sub): AccountIdExtractor,
    Path(set): Path<String>,
    Path(object): Path<String>,
    headers: HeaderMap,
) -> Response<String> {
    let back = String::from(crate::app::util::S3_DEFAULT_CLIENT);
    read_ns(ctx, back, set, object, sub, headers.get(REFERER)).await
}

pub async fn backend_read(
    State(ctx): State<Arc<AppContext>>,
    AccountIdExtractor(sub): AccountIdExtractor,
    Path(back): Path<String>,
    Path(set): Path<String>,
    Path(object): Path<String>,
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
    let zobj = AuthzObject::new(&vec!["sets", &set]);
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
                    format!("Error reading an object by set: {}", err.to_string()),
                ),
                Ok(_) => {
                    let bucket = set_s.bucket().to_string();
                    let object = s3_object(set_s.label(), &object);

                    match s3.presigned_url("GET", &bucket, &object) {
                        Ok(uri) => redirect(uri),
                        Err(err) => wrap_error(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            format!("Error reading an object by set: {}", err.to_string()),
                        ),
                    }
                }
            }
        }
        Err(err) => wrap_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error reading an object by set: {}", err.to_string()),
        ),
    }
}

fn valid_referer(
    ctx: &Arc<AppContext>,
    bucket: &str,
    referer: Option<&HeaderValue>,
) -> Result<(), Response<String>> {
    let referer = match referer {
        None => None,
        Some(r) => match r.to_str() {
            Ok(r) => Some(r),
            Err(err) => {
                return Err(wrap_error(
                    StatusCode::BAD_REQUEST,
                    format!("Error reading 'REFERER' header: {}", err.to_string()),
                ))
            }
        },
    };

    match ctx.aud_estm.estimate(bucket) {
        Ok(aud) => match ctx.audiences_settings.get(aud) {
            Some(aud_settings) => if !aud_settings.valid_referer(referer.as_deref()) {
                return Err(wrap_error(
                    StatusCode::FORBIDDEN,
                    "Error reading an object using Set API: Invalid request".to_string(),
                ));
            }
            None => {
                return Err(wrap_error(
                    StatusCode::NOT_FOUND,
                    format!("Error reading an object using Set API: Audience settings for bucket '{}' not found", &bucket),
                ));
            }
        }
        Err(err) =>
            return Err(wrap_error(
                StatusCode::NOT_FOUND,
                format!("Error reading an object using Set API: Audience estimate for bucket '{}' not found, err = {}", &bucket, err),
            ))
    }

    Ok(())
}

fn parse_action(method: &str) -> anyhow::Result<&str> {
    match method {
        "HEAD" => Ok("read"),
        "GET" => Ok("read"),
        "PUT" => Ok("update"),
        "DELETE" => Ok("delete"),
        _ => Err(anyhow!("invalid method = {}", method)),
    }
}

fn s3_object(set: &str, object: &str) -> String {
    format!("{set}.{object}", set = set, object = object)
}

fn redirect(uri: String) -> Response<String> {
    Response::builder()
        .header("location", uri)
        .header("Timing-Allow-Origin", "*")
        .status(StatusCode::SEE_OTHER)
        .body(String::default())
        .unwrap()
}

fn wrap_error(status: StatusCode, msg: String) -> Response<String> {
    error!("{}", msg);
    Response::builder().status(status).body(msg).unwrap()
}
