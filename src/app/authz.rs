use app::config::Authz;
use failure::{err_msg, Error};

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

type Action<'a> = &'a str;

#[derive(Debug, Serialize)]
struct Request<'a> {
    subject: &'a Entity<'a>,
    object: &'a Entity<'a>,
    action: Action<'a>,
}

pub(crate) trait Authorization {
    fn authorize(&self, subject: &Entity, object: &Entity, action: Action) -> Result<(), Error>;
}

#[derive(Debug)]
pub(crate) struct Trusted {}

impl Trusted {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Authorization for Trusted {
    fn authorize(&self, _subect: &Entity, _object: &Entity, _action: Action) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct HttpClient {
    pub(crate) uri: String,
}

impl HttpClient {
    pub(crate) fn new(uri: &str) -> Self {
        Self {
            uri: uri.to_string(),
        }
    }
}

impl Authorization for HttpClient {
    fn authorize(&self, subject: &Entity, object: &Entity, action: Action) -> Result<(), Error> {
        use reqwest;

        let req = Request {
            subject,
            object,
            action,
        };
        let client = reqwest::Client::new();
        let resp: Vec<String> = client.post(&self.uri).json(&req).send()?.json()?;

        if !resp.contains(&action.to_string()) {
            return Err(err_msg("access is forbidden"));
        }

        Ok(())
    }
}

pub(crate) fn client(config: &Authz) -> Box<Authorization> {
    match config.uri {
        Some(ref val) => Box::new(HttpClient::new(val)),
        None => Box::new(Trusted::new()),
    }
}
