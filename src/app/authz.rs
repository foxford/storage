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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TrustedClient {}

impl Authorization for TrustedClient {
    fn authorize(&self, _subject: &Entity, _object: &Entity, _action: Action) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HttpClient {
    pub(crate) uri: String,
    pub(crate) token: String,
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
        let resp: Vec<String> = client
            .post(&self.uri)
            .bearer_auth(&self.token)
            .json(&req)
            .send()?
            .json()?;

        if !resp.contains(&action.to_string()) {
            return Err(err_msg(format!(
                "{:?} access to {:?} is forbidden for {:?}",
                action, object, subject
            )));
        }

        Ok(())
    }
}
