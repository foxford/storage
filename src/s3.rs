use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use anyhow::{Context, Result};
use rusoto_core::{credential::AwsCredentials, signature::SignedRequest, Region};
use url::Url;

use crate::app::util::ProxyHost;

const DEFAULT_COUNTRY_CODE: &str = "default";

#[derive(Debug)]
pub struct Client {
    credentials: AwsCredentials,
    region: Region,
    expires_in: Duration,
    proxy_hosts: Option<BTreeMap<String, Vec<String>>>,
    counter: AtomicUsize,
}

impl Client {
    pub fn new(
        key: &str,
        secret: &str,
        region: &str,
        endpoint: &str,
        expires_in: Duration,
    ) -> Client {
        let region = Region::Custom {
            name: region.to_string(),
            endpoint: endpoint.to_string(),
        };
        let credentials = AwsCredentials::new(key, secret, None, None);

        Self {
            credentials,
            region,
            expires_in,
            proxy_hosts: None,
            counter: AtomicUsize::new(0),
        }
    }

    pub fn set_proxy_hosts(&mut self, hosts: &[ProxyHost]) -> &mut Self {
        let mut resulting_hosts: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for host in hosts {
            let country_code = host
                .country
                .clone()
                .unwrap_or(DEFAULT_COUNTRY_CODE.to_string());
            match host.alias_range_upper_bound {
                Some(upper_bound) => {
                    for alias_index in 1..=upper_bound {
                        let host_uri = format!("{}.{}", alias_index, host.base);
                        if let Some(hosts) = resulting_hosts.get_mut(&country_code) {
                            hosts.push(host_uri);
                        } else {
                            resulting_hosts.insert(country_code.clone(), vec![host_uri]);
                        }
                    }
                }
                None => {
                    let host_uri = host.base.to_owned();
                    if let Some(hosts) = resulting_hosts.get_mut(&country_code) {
                        hosts.push(host_uri);
                    } else {
                        resulting_hosts.insert(country_code, vec![host_uri]);
                    }
                }
            }
        }

        self.proxy_hosts = Some(resulting_hosts);
        self
    }

    pub fn create_request(&self, method: &str, bucket: &str, object: &str) -> SignedRequest {
        let uri = format!("/{bucket}/{object}", bucket = bucket, object = object);
        SignedRequest::new(method, "s3", &self.region, &uri)
    }

    fn get_proxy_hosts(&self, country: Option<&String>) -> Option<&Vec<String>> {
        let default = DEFAULT_COUNTRY_CODE.to_string();
        let country_code = country.unwrap_or(&default);
        self.proxy_hosts.as_ref().and_then(|c| c.get(country_code))
    }

    pub fn sign_request(
        &self,
        req: &mut SignedRequest,
        country: Option<&String>,
    ) -> Result<String> {
        let url = req.generate_presigned_url(&self.credentials, &self.expires_in, false);

        if let Some(proxy_hosts) = self.get_proxy_hosts(country) {
            let mut parsed_url = Url::parse(&url).context("failed to parse generated uri")?;

            let idx = self.counter.fetch_add(1, Ordering::Acquire) % proxy_hosts.len();
            parsed_url
                .set_host(proxy_hosts.get(idx).map(|h| h.as_str()))
                .context("failed to set proxy backend")?;

            Ok(parsed_url.to_string())
        } else {
            Ok(url)
        }
    }

    pub fn presigned_url(
        &self,
        country: Option<&String>,
        method: &str,
        bucket: &str,
        object: &str,
    ) -> Result<String> {
        self.sign_request(&mut self.create_request(method, bucket, object), country)
    }
}
