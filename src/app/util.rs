use anyhow::{anyhow, Result};
use radix_trie::Trie;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    sync::Arc,
};
use svc_authn::{AccountId, Authenticable};

use crate::db::{Bucket, Set};
use crate::s3::Client;

////////////////////////////////////////////////////////////////////////////////

pub type S3Clients = BTreeMap<String, Arc<Client>>;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize)]
pub struct BackendConfig(BTreeMap<String, BackendConfigItem>);

#[derive(Clone, Debug, Deserialize)]
pub struct BackendConfigItem {
    proxy_hosts: HashMap<String, ProxyHost>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProxyHost {
    pub base: String,
    pub alias_range_upper_bound: Option<usize>,
}

////////////////////////////////////////////////////////////////////////////////

pub fn read_s3_config(config: &BackendConfig) -> Result<S3Clients> {
    let mut acc = S3Clients::new();

    for (back, config) in config.0.iter() {
        read_s3(back, &format!("{}_", back.to_uppercase()), config, &mut acc);
    }

    Ok(acc)
}

fn read_s3(back: &str, prefix: &str, item: &BackendConfigItem, acc: &mut S3Clients) {
    use std::env::var;
    let key = var(format!("{}AWS_ACCESS_KEY_ID", prefix))
        .unwrap_or_else(|_| panic!("{}AWS_ACCESS_KEY_ID must be specified", prefix));
    let secret = var(format!("{}AWS_SECRET_ACCESS_KEY", prefix))
        .unwrap_or_else(|_| panic!("{}AWS_SECRET_ACCESS_KEY must be specified", prefix));
    let endpoint = var(format!("{}AWS_ENDPOINT", prefix))
        .unwrap_or_else(|_| panic!("{}AWS_ENDPOINT must be specified", prefix));
    let region = var(format!("{}AWS_REGION", prefix))
        .unwrap_or_else(|_| panic!("{}AWS_REGION must be specified", prefix));

    let client = Client::new(
        &key,
        &secret,
        &region,
        &endpoint,
        ::std::time::Duration::from_secs(300),
        item.proxy_hosts.clone(),
    );

    acc.insert(back.to_owned(), Arc::new(client));
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct S3SignedRequestBuilder {
    method: Option<String>,
    bucket: Option<String>,
    object: Option<String>,
    headers: BTreeMap<String, String>,
}

impl S3SignedRequestBuilder {
    pub fn new() -> Self {
        Self {
            method: None,
            bucket: None,
            object: None,
            headers: BTreeMap::new(),
        }
    }

    pub fn method(self, value: &str) -> Self {
        Self {
            method: Some(value.to_string()),
            ..self
        }
    }

    pub fn bucket(self, value: &str) -> Self {
        Self {
            bucket: Some(value.to_string()),
            ..self
        }
    }

    pub fn object(self, value: &str) -> Self {
        Self {
            object: Some(value.to_string()),
            ..self
        }
    }

    pub fn add_header(self, key: &str, value: &str) -> Self {
        let mut headers = self.headers;
        headers.insert(key.to_string(), value.to_string());
        Self { headers, ..self }
    }

    pub fn build(self, client: &Client, country: Option<String>) -> Result<String> {
        let mut req = client.create_request(
            &self
                .method
                .ok_or_else(|| anyhow!("Error building a signed request. missing method."))?,
            &self
                .bucket
                .ok_or_else(|| anyhow!("Error building a signed request. missing bucket"))?,
            &self
                .object
                .ok_or_else(|| anyhow!("Error building a signed request. missing object"))?,
        );
        for (key, val) in self.headers {
            req.add_header(&key, &val);
        }

        client
            .sign_request(&mut req, country)
            .map_err(|err| anyhow!("Error building a signed request. {}", &err.to_string()))
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct AudienceEstimator {
    inner: Trie<String, String>,
}

impl AudienceEstimator {
    pub fn new(config: &svc_authz::ConfigMap) -> Self {
        let mut inner = Trie::new();
        config.iter().for_each(|(key, _val)| {
            let rkey = key.split('.').rev().collect::<Vec<&str>>().join(".");
            inner.insert(rkey, key.clone());
        });
        Self { inner }
    }

    pub fn estimate(&self, bucket: &str) -> Result<&str> {
        let rbucket = bucket.split('.').rev().collect::<Vec<&str>>().join(".");
        self.inner
            .get_ancestor_value(&rbucket)
            .map(|aud| aud.as_ref())
            .ok_or_else(|| anyhow!("Error estimating an audience of the bucket('{}')", bucket))
    }

    pub fn parse_set(&self, value: &str) -> Result<Set> {
        let parts: Vec<&str> = value.split("::").collect();
        if parts.len() < 2 {
            return Err(anyhow!("Error parsing a set('{}')", value));
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
