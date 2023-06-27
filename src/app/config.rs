use serde::Deserialize;
use std::{collections::BTreeMap, net::SocketAddr};
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub id: svc_authn::AccountId,
    pub backend: crate::app::util::BackendConfig,
    pub authn: svc_authn::jose::ConfigMap,
    pub authz: svc_authz::ConfigMap,
    pub http: HttpConfig,
    pub audiences_settings: BTreeMap<String, AudienceSettings>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    pub listener_address: SocketAddr,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AudienceSettings {
    allowed_referers: Option<Vec<String>>,
}

impl AppConfig {
    pub fn load() -> Result<AppConfig, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("App"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()
            .and_then(|c| c.try_deserialize::<AppConfig>())
    }
}

impl AudienceSettings {
    pub fn valid_referer(&self, referer: Option<&str>) -> bool {
        match (&self.allowed_referers, referer) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(referers), Some(referer)) => {
                if let Some(host) = Url::parse(referer)
                    .ok()
                    .and_then(|u| u.host().map(|h| h.to_string()))
                {
                    referers.iter().any(|r| {
                        if r.starts_with('*') {
                            host.ends_with(&r.replace('*', ""))
                        } else {
                            *r == host
                        }
                    })
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_referer_no_refs() {
        let s = AudienceSettings {
            allowed_referers: None,
        };
        assert!(s.valid_referer(None));
        assert!(s.valid_referer(Some("foobar")));
    }

    #[test]
    fn valid_referer_no_referer() {
        let s = AudienceSettings {
            allowed_referers: Some(vec!["foo".into(), "bar".into(), "baz".into()]),
        };
        assert!(!s.valid_referer(None));
        assert!(s.valid_referer(Some("http://foo")));
        assert!(s.valid_referer(Some("https://foo")));
        assert!(!s.valid_referer(Some("https://quux")));
    }

    #[test]
    fn valid_referer_mask() {
        let s = AudienceSettings {
            allowed_referers: Some(vec!["*.foo".into()]),
        };
        assert!(!s.valid_referer(None));
        assert!(s.valid_referer(Some("http://baz.foo")));
        assert!(s.valid_referer(Some("https://bar.foo")));
        assert!(!s.valid_referer(Some("http://qwe.quux")));
        assert!(!s.valid_referer(Some("http://foo")));
    }
}
