use crate::{
    app::error::{Error, ErrorKind},
    s3::DEFAULT_COUNTRY_CODE,
};
use axum::{
    async_trait,
    extract::{Extension, FromRequestParts},
    http::request::Parts,
};
use axum_client_ip::InsecureClientIp;
use maxminddb::{geoip2::Country, Reader};
use std::sync::Arc;
use tracing::{error, field, Span};

/// Extracts iso code of country from ip address.
pub struct CountryExtractor(pub String);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CountryExtractor {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let Extension(maxmind) = parts
            .extract::<Extension<Arc<Reader<Vec<u8>>>>>()
            .await
            .ok()
            .ok_or(Error::new(ErrorKind::MissingMaxmind, None))?;

        let InsecureClientIp(ip_address) =
            match InsecureClientIp::from_request_parts(parts, state).await {
                Ok(ip) => ip,
                Err((_, err)) => {
                    error!("error retrieve ip address: {}", err);
                    return Ok(Self(DEFAULT_COUNTRY_CODE.to_string()));
                }
            };

        Span::current().record("ip_address", &field::display(&ip_address));

        let country: String = match maxmind.lookup::<Country>(ip_address) {
            Ok(country) => country
                .country
                .and_then(|c| c.iso_code)
                .map(|c| c.to_string())
                .unwrap_or_else(|| DEFAULT_COUNTRY_CODE.to_string()),
            Err(err) => {
                error!("maxminddb error: {}", err);
                DEFAULT_COUNTRY_CODE.to_string()
            }
        };

        Ok(Self(country))
    }
}
