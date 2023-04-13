use std::collections::BTreeMap;

use url::Url;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) id: svc_authn::AccountId,
    pub(crate) backend: Option<crate::app::util::BackendConfig>,
    pub(crate) authn: svc_authn::jose::ConfigMap,
    pub(crate) authz: svc_authz::ConfigMap,
    pub(crate) http: crate::app::HttpConfig,
    pub(crate) audiences_settings: BTreeMap<String, AudienceSettings>,
}

pub(crate) fn load() -> Result<Config, config::ConfigError> {
    let mut parser = config::Config::default();
    parser.merge(config::File::with_name("App"))?;
    parser.merge(config::Environment::with_prefix("APP").separator("__"))?;
    parser.try_into::<Config>()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AudienceSettings {
    allowed_referers: Option<Vec<String>>,
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
