use axum::{
    async_trait,
    extract::{Extension, FromRequestParts, Json},
    http::{request::Parts, StatusCode},
};
use std::sync::Arc;
use svc_agent::AccountId;
use svc_authn::jose::ConfigMap as AuthnConfig;
use svc_authn::token::jws_compact::extract::decode_jws_compact_with_config;
use svc_error::Error;
use svc_utils::extractors::AccountIdExtractor;
use tracing::{field, Span};

/// Extracts `AccountId` from "Authorization: Bearer ..." header or query string
pub struct AccessTokenExtractor(pub AccountId);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AccessTokenExtractor {
    type Rejection = (StatusCode, Json<Error>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        if let Ok(AccountIdExtractor(account_id)) =
            AccountIdExtractor::from_request_parts(parts, state).await
        {
            return Ok(Self(account_id));
        }

        let Extension(authn) = parts
            .extract::<Extension<Arc<AuthnConfig>>>()
            .await
            .ok()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(Error::new(
                    "no_authn_config",
                    "No authn config",
                    StatusCode::UNAUTHORIZED,
                )),
            ))?;

        let access_token = url::form_urlencoded::parse(parts.uri.query().unwrap_or("").as_bytes())
            .find(|(key, _)| key == "access_token")
            .map(|(_, val)| val)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(Error::new(
                    "invalid_authentication",
                    "Invalid authentication",
                    StatusCode::UNAUTHORIZED,
                )),
            ))?;

        let claims = decode_jws_compact_with_config::<String>(&access_token, &authn)
            .map_err(|e| {
                let err = e.to_string();
                (
                    StatusCode::UNAUTHORIZED,
                    Json(Error::new(
                        "invalid_authentication",
                        &err,
                        StatusCode::UNAUTHORIZED,
                    )),
                )
            })?
            .claims;
        let account_id = AccountId::new(claims.subject(), claims.audience());

        Span::current().record("account_id", &field::display(&account_id));

        Ok(Self(account_id))
    }
}
