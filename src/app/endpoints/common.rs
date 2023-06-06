use crate::app::{
    context::AppContext,
    error::{Error, ErrorKind},
};
use axum::response::{IntoResponse, Response};
use http::header::HeaderValue;
use std::sync::Arc;
use tracing::error;

pub fn valid_referer(
    ctx: &Arc<AppContext>,
    bucket: &str,
    referer: Option<&HeaderValue>,
) -> Result<(), Response> {
    let referer = match referer {
        None => None,
        Some(r) => match r.to_str() {
            Ok(r) => Some(r),
            Err(err) => return Err(wrap_error(ErrorKind::RefererError, err.to_string())),
        },
    };

    match ctx.aud_estm.estimate(bucket) {
        Ok(aud) => match ctx.audiences_settings.get(aud) {
            Some(aud_settings) => if !aud_settings.valid_referer(referer) {
                return Err(wrap_error(ErrorKind::RefererError, "Error reading 'REFERER' header".to_string()));
            }
            None => {
                return Err(wrap_error(
                    ErrorKind::MissingAudienceSetting,
                    format!("Error reading an object using Set API: Audience settings for bucket '{}' not found", &bucket)
                ));
            }
        }
        Err(err) =>
            return Err(wrap_error(
                ErrorKind::MissingAudienceSetting,
                format!("Error reading an object using Set API: Audience estimate for bucket '{}' not found, err = {}", &bucket, err)
            )),
    }

    Ok(())
}

pub fn s3_object(set: &str, object: &str) -> String {
    format!("{set}.{object}")
}

pub fn wrap_error(kind: ErrorKind, msg: String) -> Response {
    use anyhow::anyhow;

    error!("{}", msg);
    Error::new(kind, Some(anyhow!(msg))).into_response()
}
