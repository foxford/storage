use radix_trie::Trie;
use std::collections::BTreeMap;
use tower_web::extract::{Context, Extract, Immediate};
use tower_web::util::BufStream;
use tower_web::Error;

use crate::s3::Client;

////////////////////////////////////////////////////////////////////////////////

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
        let unproc_error = || {
            Error::builder()
                .kind(
                    "s3_signed_request_builder_error",
                    "Error building a signed request",
                )
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        };

        let mut req = client.create_request(
            &self
                .method
                .ok_or_else(|| unproc_error().detail("missing method").build())?,
            &self
                .bucket
                .ok_or_else(|| unproc_error().detail("missing bucket").build())?,
            &self
                .object
                .ok_or_else(|| unproc_error().detail("missing object").build())?,
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

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct AudienceEstimator {
    inner: Trie<String, String>,
}

impl AudienceEstimator {
    pub(crate) fn new(config: &svc_authz::ConfigMap) -> Self {
        let mut inner = Trie::new();
        config.iter().for_each(|(key, _val)| {
            let rkey = key.split('.').rev().collect::<Vec<&str>>().join(".");
            inner.insert(rkey, key.clone());
        });
        Self { inner }
    }

    pub(crate) fn estimate(&self, bucket: &str) -> Result<&str, Error> {
        let unproc_error = || {
            Error::builder()
                .kind(
                    "audience_estimator_error",
                    "Error estimating an audience of the bucket",
                )
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        };

        let rbucket = bucket.split('.').rev().collect::<Vec<&str>>().join(".");
        self.inner
            .get_ancestor_value(&rbucket)
            .map(|aud| aud.as_ref())
            .ok_or_else(|| {
                unproc_error()
                    .detail(&format!("invalid bucket = '{}'", bucket))
                    .build()
            })
    }
}
