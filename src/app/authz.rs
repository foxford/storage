use app::authn::Subject;
use app::config::Authz;

type Object<'a> = Vec<&'a str>;
type Action<'a> = &'a str;

pub(crate) trait Authorization {
    fn authorize(self: &Self, sub: &Subject, object: &Object, action: Action) -> Result<(), ()>;
}

#[derive(Debug)]
pub(crate) struct Trusted {}

impl Trusted {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Authorization for Trusted {
    fn authorize(self: &Self, _sub: &Subject, _object: &Object, _action: Action) -> Result<(), ()> {
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
    fn authorize(self: &Self, _sub: &Subject, _object: &Object, _action: Action) -> Result<(), ()> {
        unimplemented!()
    }
}

pub(crate) fn client(config: &Authz) -> Box<Authorization> {
    match config.uri {
        Some(ref val) => Box::new(HttpClient::new(val)),
        None => Box::new(Trusted::new()),
    }
}
