use failure::{err_msg, Error};
use std::collections::BTreeMap;
use tool::s3::Client;
use tower_web::extract::{Context, Extract, Immediate};
use tower_web::util::BufStream;

#[derive(Debug)]
pub(crate) struct S3SignedRequestBuilder {
    method: Option<String>,
    bucket: Option<String>,
    object: Option<String>,
    headers: BTreeMap<String, String>,
}

impl S3SignedRequestBuilder {
    pub(crate) fn new() -> Self {
        Self {
            method: None,
            bucket: None,
            object: None,
            headers: BTreeMap::new(),
        }
    }

    pub(crate) fn method(self, value: &str) -> Self {
        Self {
            method: Some(value.to_string()),
            ..self
        }
    }

    pub(crate) fn bucket(self, value: &str) -> Self {
        Self {
            bucket: Some(value.to_string()),
            ..self
        }
    }

    pub(crate) fn object(self, value: &str) -> Self {
        Self {
            object: Some(value.to_string()),
            ..self
        }
    }

    pub(crate) fn add_header(self, key: &str, value: &str) -> Self {
        let mut headers = self.headers;
        headers.insert(key.to_string(), value.to_string());
        Self {
            headers: headers,
            ..self
        }
    }

    pub(crate) fn build(self, client: &Client) -> Result<String, Error> {
        let mut req = client.create_request(
            &self.method.ok_or_else(|| err_msg("method is required"))?,
            &self.bucket.ok_or_else(|| err_msg("bucket is required"))?,
            &self.object.ok_or_else(|| err_msg("object is required"))?,
        );
        for (key, val) in self.headers {
            req.add_header(&key, &val);
        }

        let uri = client.sign_request(&mut req);
        Ok(uri)
    }
}

impl<B: BufStream> Extract<B> for S3SignedRequestBuilder {
    type Future = Immediate<S3SignedRequestBuilder>;

    fn extract(context: &Context) -> Self::Future {
        use tower_web::extract::Error;

        let mut builder = S3SignedRequestBuilder::new();
        let headers = context.request().headers();
        for (key, val) in headers {
            match val.to_str() {
                Ok(val) => builder = builder.add_header(key.as_str(), val),
                Err(err) => return Immediate::err(Error::invalid_argument(&err)),
            }
        }
        Immediate::ok(builder)
    }
}
