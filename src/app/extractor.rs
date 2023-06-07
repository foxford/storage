use std::sync::Arc;

use axum::{
    async_trait,
    extract::{Extension, FromRequestParts, Json},
    http::{request::Parts, StatusCode},
};
use svc_agent::AccountId;
use svc_authn::jose::ConfigMap as AuthnConfig;
use svc_authn::token::jws_compact::extract::decode_jws_compact_with_config;
use svc_error::Error;
use tracing::{error, field, Span};

/// Extracts `AccountId` from "Authorization: Bearer ..." headers.
pub struct AccountIdExtractor(pub AccountId);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AccountIdExtractor {
    type Rejection = (StatusCode, Json<Error>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
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

        error!(
            "Authorization header: {:?}",
            parts.headers.get("Authorization"),
        );
        error!(
            "headers: {:?}",
            parts.headers,
        );
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.get("Bearer ".len()..))
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(Error::new(
                    "invalid_authentication",
                    "Invalid authentication",
                    StatusCode::UNAUTHORIZED,
                )),
            ))?;

        error!("auth_header: {}", auth_header);
        let claims = decode_jws_compact_with_config::<String>(auth_header, &authn)
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
