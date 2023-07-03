use crate::app::error::{Error, ErrorKind};
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
pub struct CountryExtractor(pub Option<String>);

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

        let Ok(InsecureClientIp(ip_address)) = InsecureClientIp::from_request_parts(parts, state).await else {
            error!("error retrieve ip address");
            return Ok(Self(None));
        };

        Span::current().record("ip_address", &field::display(&ip_address));

        let country: Option<String> = match maxmind.lookup::<Country>(ip_address) {
            Ok(country) => country
                .country
                .and_then(|c| c.iso_code)
                .map(|c| c.to_string()),
            Err(err) => {
                error!("maxmind db error: {}", err);
                None
            }
        };

        Ok(Self(country))
    }
}
