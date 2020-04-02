use crate::tower_web::Error;
use radix_trie::Trie;
use std::collections::BTreeMap;
use std::ops::Deref;
use svc_authn::{AccountId, Authenticable};

use crate::db::{Bucket, Set};
use crate::s3::Client;

////////////////////////////////////////////////////////////////////////////////

pub(crate) const S3_DEFAULT_CLIENT: &str = "default";
pub(crate) type S3Clients = BTreeMap<String, ::std::sync::Arc<crate::s3::Client>>;

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn read_s3_config(backends: &Vec<String>) -> S3Clients {
    let mut acc = S3Clients::new();

    if let Some(back) = backends.first() {
        read_s3(
            &String::from(S3_DEFAULT_CLIENT),
            &format!("{}_", back.to_uppercase()),
            &mut acc,
        );
        for back in backends {
            read_s3(back, &format!("{}_", back.to_uppercase()), &mut acc);
        }
    } else {
        read_s3(&String::from(S3_DEFAULT_CLIENT), "", &mut acc);
    }

    acc
}

fn read_s3(back: &str, prefix: &str, acc: &mut S3Clients) {
    use std::env::var;
    let key = var(&format!("{}AWS_ACCESS_KEY_ID", prefix))
        .expect(&format!("{}AWS_ACCESS_KEY_ID must be specified", prefix));
    let secret = var(&format!("{}AWS_SECRET_ACCESS_KEY", prefix)).expect(&format!(
        "{}AWS_SECRET_ACCESS_KEY must be specified",
        prefix
    ));
    let endpoint = var(&format!("{}AWS_ENDPOINT", prefix))
        .expect(&format!("{}AWS_ENDPOINT must be specified", prefix));
    let region = var(&format!("{}AWS_REGION", prefix))
        .expect(&format!("{}AWS_REGION must be specified", prefix));

    let client = crate::s3::Client::new(
        &key,
        &secret,
        &region,
        &endpoint,
        ::std::time::Duration::from_secs(300),
    );

    acc.insert(back.to_owned(), ::std::sync::Arc::new(client));
}

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

    pub(crate) fn parse_bucket(&self, value: &str) -> Result<Bucket, Error> {
        self.estimate(value)
            .map(|audience| Bucket::new(Self::bucket_label(value, audience), audience))
    }

    pub(crate) fn parse_set(&self, value: &str) -> Result<Set, Error> {
        let unproc_error = || {
            Error::builder()
                .kind("audience_estimator_parsing_error", "Error parsing a set")
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        };

        let parts: Vec<&str> = value.split("::").collect();
        if parts.len() < 2 {
            return Err(unproc_error().detail(&format!("set = '{}'", value)).build());
        }

        let bucket_value = parts[0];
        let label = parts[1];
        self.estimate(bucket_value).map(|audience| {
            let bucket = Bucket::new(Self::bucket_label(bucket_value, audience), audience);
            Set::new(label, bucket)
        })
    }

    fn bucket_label<'a>(bucket: &'a str, audience: &str) -> &'a str {
        let (val, _) = bucket.split_at(bucket.len() - (audience.len() + 1));
        val
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Subject {
    inner: AccountId,
}

impl Subject {
    pub fn new(inner: AccountId) -> Self {
        Self { inner }
    }
}

impl Deref for Subject {
    type Target = AccountId;

    fn deref(&self) -> &AccountId {
        &self.inner
    }
}

impl Authenticable for Subject {
    fn as_account_id(&self) -> &AccountId {
        &self.inner
    }
}

////////////////////////////////////////////////////////////////////////////////

mod jose {
    use svc_authn::jose::Claims;

    use super::Subject;
    use svc_authn::AccountId;

    impl From<Claims<String>> for Subject {
        fn from(value: Claims<String>) -> Self {
            Self::new(AccountId::new(value.subject(), value.audience()))
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

mod tower_web {
    use super::{S3SignedRequestBuilder, Subject};

    mod extract {
        use http::StatusCode;
        use tower_web::extract::{Context, Error, Extract, Immediate};
        use tower_web::util::BufStream;

        use super::{S3SignedRequestBuilder, Subject};
        use svc_authn::jose::ConfigMap;
        use svc_authn::token::jws_compact::extract::{
            decode_jws_compact_with_config, extract_jws_compact,
        };

        impl<B: BufStream> Extract<B> for S3SignedRequestBuilder {
            type Future = Immediate<S3SignedRequestBuilder>;

            fn extract(context: &Context) -> Self::Future {
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

        impl<B: BufStream> Extract<B> for Subject {
            type Future = Immediate<Subject>;

            fn extract(context: &Context) -> Self::Future {
                let authn = context
                    .config::<ConfigMap>()
                    .expect("missing an authn config");

                let h = context.request().headers().get(http::header::AUTHORIZATION);
                let q = url::form_urlencoded::parse(
                    context
                        .request()
                        .uri()
                        .query()
                        .unwrap_or_else(|| "")
                        .as_bytes(),
                )
                .find(|(key, _)| key == "access_token")
                .map(|(_, val)| val);

                match (h, q) {
                    (Some(header), _) => match extract_jws_compact(header, authn) {
                        Ok(data) => Immediate::ok(data.claims.into()),
                        Err(ref err) => {
                            Immediate::err(error(&err.to_string(), StatusCode::UNAUTHORIZED))
                        }
                    },
                    (_, Some(token)) => {
                        match decode_jws_compact_with_config::<String>(&token, authn) {
                            Ok(data) => Immediate::ok(data.claims.into()),
                            Err(ref err) => {
                                Immediate::err(error(&err.to_string(), StatusCode::UNAUTHORIZED))
                            }
                        }
                    }
                    (None, None) => {
                        Immediate::err(error("missing authentication token", StatusCode::FORBIDDEN))
                    }
                }
            }
        }

        fn error(detail: &str, status: StatusCode) -> Error {
            let mut err = tower_web::Error::new(
                "authn_error",
                "Error processing the authentication token",
                status,
            );
            err.set_detail(detail);
            err.into()
        }
    }
}
