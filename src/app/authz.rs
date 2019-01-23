use tower_web::{Error, ErrorBuilder};

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
pub(crate) struct Request<'a> {
    subject: &'a Entity<'a>,
    object: &'a Entity<'a>,
    action: Action<'a>,
}

impl<'a> Request<'a> {
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

pub(crate) trait Authorization {
    fn authorize(&self, req: &Request) -> Result<(), Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TrustedClient {}

impl Authorization for TrustedClient {
    fn authorize(&self, _req: &Request) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HttpClient {
    pub(crate) uri: String,
    pub(crate) token: String,
}

impl Authorization for HttpClient {
    fn authorize(&self, req: &Request) -> Result<(), Error> {
        use reqwest;

        let client = reqwest::Client::new();
        let resp: Vec<String> = client
            .post(&self.uri)
            .bearer_auth(&self.token)
            .json(&req)
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

        if !resp.contains(&req.action()) {
            return Err(error()
                .detail(&format!("action = {} is not allowed", &req.action()))
                .build());
        }

        Ok(())
    }
}

fn error() -> ErrorBuilder {
    Error::builder()
        .kind("authz_error", "Access is forbidden")
        .status(http::StatusCode::FORBIDDEN)
}
