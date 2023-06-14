use crate::app::context::AppContext;
use http::{header::HeaderValue, Response, StatusCode};
use std::sync::Arc;
use tracing::error;

pub fn valid_referer(
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
                    format!("Error reading 'REFERER' header: {}", err),
                ))
            }
        },
    };

    match ctx.aud_estm.estimate(bucket) {
        Ok(aud) => match ctx.audiences_settings.get(aud) {
            Some(aud_settings) => if !aud_settings.valid_referer(referer) {
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

pub fn s3_object(set: &str, object: &str) -> String {
    format!("{set}.{object}", set = set, object = object)
}

pub fn wrap_error(status: StatusCode, msg: String) -> Response<String> {
    error!("{}", msg);
    Response::builder().status(status).body(msg).unwrap()
}
