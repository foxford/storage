use config;
use jose::Algorithm;
use std::collections::HashMap;
use std::time::Duration;
use tower_web::middleware::cors::AllowedOrigins;

mod parse;

#[derive(Debug, Deserialize)]
pub struct Authn {
    pub audience: String,
    #[serde(deserialize_with = "parse::algorithm")]
    pub algorithm: Algorithm,
    #[serde(deserialize_with = "parse::file")]
    pub key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Authz {
    pub uri: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Namespaces {
    pub app: String,
}

#[derive(Debug, Deserialize)]
pub struct Cors {
    #[serde(deserialize_with = "parse::allowed_origins")]
    #[serde(default)]
    pub allow_origins: AllowedOrigins,
    #[serde(deserialize_with = "parse::duration")]
    #[serde(default)]
    pub max_age: Duration,
}

pub type AuthnMap = HashMap<String, Authn>;
pub type AuthzMap = HashMap<String, Authz>;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub authn: AuthnMap,
    pub authz: AuthzMap,
    pub namespaces: Namespaces,
    pub cors: Cors,
}

pub(crate) fn load() -> Result<Config, config::ConfigError> {
    let mut parser = config::Config::default();
    parser.merge(config::File::with_name("App"))?;
    parser.merge(config::Environment::with_prefix("APP").separator("__"))?;
    parser.try_into::<Config>()
}
