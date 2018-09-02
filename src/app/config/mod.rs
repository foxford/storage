use config;
use std::time::Duration;
use tower_web::middleware::cors::AllowedOrigins;

mod parse;

#[derive(Debug, Deserialize)]
pub struct Cors {
    #[serde(deserialize_with = "parse::allowed_origins")]
    #[serde(default)]
    pub allow_origins: AllowedOrigins,
    #[serde(deserialize_with = "parse::duration")]
    #[serde(default)]
    pub max_age: Duration,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub cors: Cors,
}

pub(crate) fn load() -> Result<Config, config::ConfigError> {
    let mut parser = config::Config::default();
    parser.merge(config::File::with_name("App"))?;
    parser.merge(config::Environment::with_prefix("APP").separator("__"))?;
    parser.try_into::<Config>()
}
