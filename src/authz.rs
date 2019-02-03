use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fmt;
use tower_web::{Error, ErrorBuilder};

////////////////////////////////////////////////////////////////////////////////

pub(crate) trait Authorize: Sync + Send {
    fn authorize(&self, intent: &Intent) -> Result<(), Error>;
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) type ConfigMap = HashMap<String, Config>;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum Config {
    Trusted(TrustedConfig),
    Http(HttpConfig),
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) type Client = Box<dyn Authorize>;

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client")
    }
}

#[derive(Debug)]
pub(crate) struct ClientMap {
    inner: HashMap<String, Client>,
}

impl ClientMap {
    pub(crate) fn authorize(&self, audience: &str, intent: &Intent) -> Result<(), Error> {
        let client = self.inner.get(audience).ok_or_else(|| {
            let detail = format!("no authz configuration for the audience = {}", audience);
            error().detail(&detail).build()
        })?;
        client.authorize(intent)
    }
}

impl From<ConfigMap> for ClientMap {
    fn from(m: ConfigMap) -> Self {
        let inner: HashMap<String, Client> = m
            .into_iter()
            .map(|val| match val {
                (aud, Config::Trusted(config)) => (aud, config.into()),
                (aud, Config::Http(config)) => (aud, config.into()),
            })
            .collect();

        Self { inner }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize)]
pub(crate) struct Entity<'a> {
    namespace: &'a str,
    value: Vec<&'a str>,
}

impl<'a> Entity<'a> {
    pub(crate) fn new(namespace: &'a str, value: Vec<&'a str>) -> Self {
        Self { namespace, value }
    }
}

////////////////////////////////////////////////////////////////////////////////

type Action<'a> = &'a str;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize)]
pub(crate) struct Intent<'a> {
    subject: &'a Entity<'a>,
    object: &'a Entity<'a>,
    action: Action<'a>,
}

impl<'a> Intent<'a> {
    pub(crate) fn new(subject: &'a Entity<'a>, object: &'a Entity<'a>, action: Action<'a>) -> Self {
        Self {
            subject,
            object,
            action,
        }
    }

    pub(crate) fn action(&self) -> String {
        self.action.to_string()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TrustedConfig {}

impl Authorize for TrustedConfig {
    fn authorize(&self, _intent: &Intent) -> Result<(), Error> {
        Ok(())
    }
}

impl From<TrustedConfig> for Client {
    fn from(config: TrustedConfig) -> Self {
        Box::new(config)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HttpConfig {
    pub(crate) uri: String,
    pub(crate) token: String,
}

impl Authorize for HttpConfig {
    fn authorize(&self, intent: &Intent) -> Result<(), Error> {
        use reqwest;

        let client = reqwest::Client::new();
        let resp: Vec<String> = client
            .post(&self.uri)
            .bearer_auth(&self.token)
            .json(&intent)
            .send()
            .map_err(|err| {
                let detail = format!("error sending the authorization request, {}", &err);
                error().detail(&detail).build()
            })?
            .json()
            .map_err(|_| {
                error()
                    .detail("invalid format of the authorization response")
                    .build()
            })?;

        if !resp.contains(&intent.action()) {
            return Err(error()
                .detail(&format!("action = {} is not allowed", &intent.action()))
                .build());
        }

        Ok(())
    }
}

impl From<HttpConfig> for Client {
    fn from(config: HttpConfig) -> Self {
        Box::new(config)
    }
}

////////////////////////////////////////////////////////////////////////////////

fn error() -> ErrorBuilder {
    Error::builder()
        .kind("authz_error", "Access is forbidden")
        .status(http::StatusCode::FORBIDDEN)
}
