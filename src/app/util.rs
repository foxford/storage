use anyhow::format_err;
use radix_trie::Trie;
use std::collections::BTreeMap;
use std::ops::Deref;
use svc_authn::{AccountId, Authenticable};

use crate::db::{Bucket, Set};
use crate::s3::Client;
use crate::tower_web::Error;

////////////////////////////////////////////////////////////////////////////////

pub(crate) const S3_DEFAULT_CLIENT: &str = "default";
pub(crate) type S3Clients = BTreeMap<String, ::std::sync::Arc<crate::s3::Client>>;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub(crate) struct BackendConfig {
    default: String,
    alt: BTreeMap<String, AltBackendConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AltBackendConfig {
    proxy_host: Option<String>,
}

impl AltBackendConfig {
    fn new() -> Self {
        AltBackendConfig { proxy_host: None }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn read_s3_config(config: Option<&BackendConfig>) -> anyhow::Result<S3Clients> {
    let mut acc = S3Clients::new();

    if let Some(back) = config {
        read_s3(
            &String::from(S3_DEFAULT_CLIENT),
            &format!("{}_", back.default.to_uppercase()),
            back.alt
                .get(&back.default)
                .ok_or_else(|| format_err!("Missing default backend configuration"))?,
            &mut acc,
        );

        for (back, config) in back.alt.iter() {
            read_s3(back, &format!("{}_", back.to_uppercase()), config, &mut acc);
        }
    } else {
        read_s3(
            &String::from(S3_DEFAULT_CLIENT),
            "",
            &AltBackendConfig::new(),
            &mut acc,
        );
    }

    Ok(acc)
}

fn read_s3(back: &str, prefix: &str, alt: &AltBackendConfig, acc: &mut S3Clients) {
    use std::env::var;
    let key = var(&format!("{}AWS_ACCESS_KEY_ID", prefix))
        .unwrap_or_else(|_| panic!("{}AWS_ACCESS_KEY_ID must be specified", prefix));
    let secret = var(&format!("{}AWS_SECRET_ACCESS_KEY", prefix))
        .unwrap_or_else(|_| panic!("{}AWS_SECRET_ACCESS_KEY must be specified", prefix));
    let endpoint = var(&format!("{}AWS_ENDPOINT", prefix))
        .unwrap_or_else(|_| panic!("{}AWS_ENDPOINT must be specified", prefix));
    let region = var(&format!("{}AWS_REGION", prefix))
        .unwrap_or_else(|_| panic!("{}AWS_REGION must be specified", prefix));

    let mut client = crate::s3::Client::new(
        &key,
        &secret,
        &region,
        &endpoint,
        ::std::time::Duration::from_secs(300),
    );

    if let Some(ref proxy_host) = alt.proxy_host {
        client.set_proxy_host(proxy_host);
    }

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
        Self { headers, ..self }
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

        client
            .sign_request(&mut req)
            .map_err(|err| unproc_error().detail(&err.to_string()).build())
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

        use svc_authn::token::jws_compact::extract::{
            decode_jws_compact_with_config, extract_jws_compact,
        };
        use svc_authn::AccountId;

        use crate::app::config::Config;

        use super::{S3SignedRequestBuilder, Subject};

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
                let config = context.config::<Config>().expect("missing config");
                let h = context.request().headers().get(http::header::AUTHORIZATION);
                let q = url::form_urlencoded::parse(
                    context.request().uri().query().unwrap_or("").as_bytes(),
                )
                .find(|(key, _)| key == "access_token")
                .map(|(_, val)| val);

                match (h, q) {
                    (Some(header), _) => match extract_jws_compact(header, &config.authn) {
                        Ok(data) => Immediate::ok(data.claims.into()),
                        Err(ref err) => {
                            Immediate::err(error(&err.to_string(), StatusCode::UNAUTHORIZED))
                        }
                    },
                    (_, Some(token)) => {
                        match decode_jws_compact_with_config::<String>(&token, &config.authn) {
                            Ok(data) => Immediate::ok(data.claims.into()),
                            Err(ref err) => {
                                Immediate::err(error(&err.to_string(), StatusCode::UNAUTHORIZED))
                            }
                        }
                    }
                    (None, None) => {
                        let audience = config.id.audience();
                        let anonymous = AccountId::new("anonymous", audience);
                        Immediate::ok(Subject::new(anonymous))
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
