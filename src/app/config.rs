use std::collections::HashMap;

use config;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) id: svc_authn::AccountId,
    #[serde(default)]
    pub(crate) default_backend: Option<String>,
    pub(crate) backends: HashMap<String, Backend>,
    pub(crate) authn: svc_authn::jose::ConfigMap,
    pub(crate) authz: svc_authz::ConfigMap,
    pub(crate) http: crate::app::HttpConfig,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Backend {
    #[serde(default)]
    pub(crate) proxy_host: Option<String>,
}

pub(crate) fn load() -> Result<Config, config::ConfigError> {
    let mut parser = config::Config::default();
    parser.merge(config::File::with_name("App"))?;
    parser.merge(config::Environment::with_prefix("APP").separator("__"))?;
    parser.try_into::<Config>()
}
