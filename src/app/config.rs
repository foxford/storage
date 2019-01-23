use crate::app::authz;
use config;
use jsonwebtoken::Algorithm;
use std::collections::HashMap;
use std::time::Duration;
use tower_web::middleware::cors::AllowedOrigins;

#[derive(Debug, Deserialize)]
pub(crate) struct Authn {
    pub(crate) audience: String,
    #[serde(deserialize_with = "crate::serde::algorithm")]
    pub(crate) algorithm: Algorithm,
    #[serde(deserialize_with = "crate::serde::file")]
    pub(crate) key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum Authz {
    Trusted(authz::TrustedClient),
    Http(authz::HttpClient),
}

impl Authz {
    pub(crate) fn client(config: &Authz) -> Box<&authz::Authorization> {
        match config {
            Authz::Trusted(inner) => Box::new(inner),
            Authz::Http(inner) => Box::new(inner),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Namespaces {
    pub app: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Cors {
    #[serde(deserialize_with = "crate::serde::allowed_origins")]
    #[serde(default)]
    pub(crate) allow_origins: AllowedOrigins,
    #[serde(deserialize_with = "crate::serde::duration")]
    #[serde(default)]
    pub(crate) max_age: Duration,
}

pub(crate) type AuthnMap = HashMap<String, Authn>;
pub(crate) type AuthzMap = HashMap<String, Authz>;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) authn: AuthnMap,
    pub(crate) authz: AuthzMap,
    pub(crate) namespaces: Namespaces,
    pub(crate) cors: Cors,
}

pub(crate) fn load() -> Result<Config, config::ConfigError> {
    let mut parser = config::Config::default();
    parser.merge(config::File::with_name("App"))?;
    parser.merge(config::Environment::with_prefix("APP").separator("__"))?;
    parser.try_into::<Config>()
}
