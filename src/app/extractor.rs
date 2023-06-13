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
use tracing::{field, Span};

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

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.get("Bearer ".len()..));
        let access_token = url::form_urlencoded::parse(parts.uri.query().unwrap_or("").as_bytes())
            .find(|(key, _)| key == "access_token")
            .map(|(_, val)| val);

        let claims = match (auth_header, access_token) {
            (Some(token), _) => decode_jws_compact_with_config::<String>(token, &authn),
            (_, Some(token)) => decode_jws_compact_with_config::<String>(&token, &authn),
            (None, None) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(Error::new(
                        "invalid_authentication",
                        "Invalid authentication",
                        StatusCode::UNAUTHORIZED,
                    )),
                ))
            }
        }
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
