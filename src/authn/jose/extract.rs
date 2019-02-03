use crate::authn::jose::token;
use crate::authn::AccountId;
use http::header::HeaderValue;
use http::StatusCode;
use jsonwebtoken::{decode, Algorithm, TokenData, Validation};
use serde_derive::Deserialize;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use tower_web::extract::{Context, Error as ExtractError, Extract, Immediate};
use tower_web::util::BufStream;
use tower_web::{Error, ErrorBuilder};

////////////////////////////////////////////////////////////////////////////////

pub(crate) type ConfigMap = HashMap<String, Config>;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    audience: HashSet<String>,
    #[serde(deserialize_with = "crate::serde::algorithm")]
    algorithm: Algorithm,
    #[serde(deserialize_with = "crate::serde::file")]
    key: Vec<u8>,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct BearerToken {
    inner: String,
}

impl Deref for BearerToken {
    type Target = str;

    fn deref(&self) -> &str {
        &self.inner
    }
}

impl BearerToken {
    fn new(inner: &str) -> Self {
        Self {
            inner: inner.to_owned(),
        }
    }
}

impl<B: BufStream> Extract<B> for BearerToken {
    type Future = Immediate<BearerToken>;

    fn extract(context: &Context) -> Self::Future {
        match context.request().headers().get(http::header::AUTHORIZATION) {
            Some(header) => match parse_bearer_token(&header) {
                Ok(token) => Immediate::ok(BearerToken::new(token)),
                Err(err) => Immediate::err(err),
            },
            None => Immediate::err(missing_token_error().into()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<B: BufStream> Extract<B> for AccountId {
    type Future = Immediate<AccountId>;

    fn extract(context: &Context) -> Self::Future {
        let authn = context
            .config::<ConfigMap>()
            .expect("missing an authn config");
        match context.request().headers().get(http::header::AUTHORIZATION) {
            Some(header) => match extract_account_id(&header, authn) {
                Ok(sub) => Immediate::ok(sub),
                Err(err) => Immediate::err(err),
            },
            None => Immediate::err(missing_token_error().into()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

fn extract_account_id(header: &HeaderValue, authn: &ConfigMap) -> Result<AccountId, ExtractError> {
    let token = parse_bearer_token(header)?;
    let parts = parse_jws_compact(token)?;
    let config = authn.get(parts.claims.issuer()).ok_or_else(|| {
        let detail = format!(
            "issuer = {} of the authentication token is not allowed",
            parts.claims.issuer(),
        );
        ExtractError::from(error().detail(&detail).build())
    })?;

    // NOTE: we consider the token valid if its audience matches at least
    // one audience from the app config for the same issuer.
    // We can't use 'verifier.set_audience(&config.audience)' because it's
    // succeed if only all values from the app config represented in the token.
    if !config.audience.contains(parts.claims.audience()) {
        let detail = format!(
            "audience = {} of the authentication token is not allowed",
            parts.claims.audience(),
        );
        return Err(ExtractError::from(error().detail(&detail).build()));
    }

    let mut verifier = Validation::new(config.algorithm);
    verifier.validate_exp = parts.claims.expiration_time().is_some();

    decode_account_id(token, &verifier, config.key.as_ref()).map_err(|err| err.into())
}

fn parse_bearer_token(header: &HeaderValue) -> Result<&str, ExtractError> {
    let val: Vec<&str> = header
        .to_str()
        .map_err(|_| {
            ExtractError::from(
                error()
                    .detail("invalid characters in the authorization header")
                    .build(),
            )
        })?
        .split(' ')
        .collect();

    match val[..] {
        ["Bearer", ref val] => Ok(val),
        _ => Err(error()
            .detail("unsupported or invalid type of the authentication token")
            .build()
            .into()),
    }
}

pub(crate) fn decode_account_id(
    token: &str,
    verifier: &Validation,
    key: &[u8],
) -> Result<AccountId, Error> {
    let data = decode::<token::Claims>(token, key, &verifier).map_err(|err| {
        error()
            .detail(&format!(
                "verification of the authentication token failed – {}",
                &err
            ))
            .build()
    })?;

    Ok(AccountId::new(
        data.claims.subject(),
        data.claims.audience(),
    ))
}

pub(crate) fn parse_jws_compact(token: &str) -> Result<TokenData<token::Claims>, Error> {
    jsonwebtoken::dangerous_unsafe_decode(token).map_err(|_| {
        error()
            .detail("invalid claims of the authentication token")
            .build()
    })
}

////////////////////////////////////////////////////////////////////////////////

fn error() -> ErrorBuilder {
    Error::builder()
        .kind("authn_error", "Error processing the authentication token")
        .status(StatusCode::UNAUTHORIZED)
}

fn missing_token_error() -> Error {
    Error::builder()
        .kind("authn_error", "Error processing the authentication token")
        .status(StatusCode::FORBIDDEN)
        .detail("missing authentication token")
        .build()
}
