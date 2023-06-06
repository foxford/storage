use axum::{
    extract::{Path, State},
    http::header::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use http::{header::REFERER, StatusCode};
use std::sync::Arc;

use svc_authn::AccountId;
use svc_utils::extractors::AccountIdExtractor;

use super::{s3_object, valid_referer, wrap_error};
use crate::app::{
    authz::AuthzObject, context::AppContext, error::ErrorKind, maxmind::CountryExtractor,
};

pub async fn backend_read(
    State(ctx): State<Arc<AppContext>>,
    AccountIdExtractor(sub): AccountIdExtractor,
    CountryExtractor(country): CountryExtractor,
    Path(back): Path<String>,
    Path(set): Path<String>,
    Path(object): Path<String>,
    headers: HeaderMap,
) -> Response {
    read_ns(ctx, country, back, set, object, sub, headers.get(REFERER)).await
}

async fn read_ns(
    ctx: Arc<AppContext>,
    country: String,
    back: String,
    set: String,
    object: String,
    sub: AccountId,
    referer: Option<&HeaderValue>,
) -> Response {
    let zobj = AuthzObject::new(&["sets", &set]);
    let zact = "read";
    let s3 = match ctx.s3.get(&back) {
        Some(val) => val.clone(),
        None => {
            return wrap_error(
                ErrorKind::BackendNotFound,
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
                    ErrorKind::ObjectReadingError,
                    format!("Error reading an object by set: {}", err),
                ),
                Ok(_) => {
                    let bucket = set_s.bucket().to_string();
                    let object = s3_object(set_s.label(), &object);

                    match s3.presigned_url(&country, "GET", &bucket, &object) {
                        Ok(uri) => redirect(uri),
                        Err(err) => wrap_error(
                            ErrorKind::ObjectReadingError,
                            format!("Error reading an object by set: {}", err),
                        ),
                    }
                }
            }
        }
        Err(err) => wrap_error(
            ErrorKind::ObjectReadingError,
            format!("Error reading an object by set: {}", err),
        ),
    }
}

fn redirect(uri: String) -> Response {
    (
        StatusCode::SEE_OTHER,
        [("location", uri), ("Timing-Allow-Origin", "*".to_string())],
    )
        .into_response()
}

fn wrap_error(status: StatusCode, msg: String) -> Response<String> {
    error!("{}", msg);
    Response::builder().status(status).body(msg).unwrap()
}
