use std::{
    collections::{BTreeMap, HashMap},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use anyhow::{Context, Result};
use rusoto_core::{credential::AwsCredentials, signature::SignedRequest, Region};
use url::Url;

use crate::app::util::ProxyHost;

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

    pub fn set_proxy_hosts(&mut self, hosts: &HashMap<String, ProxyHost>) -> &mut Self {
        let mut proxy_hosts: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for (country, host) in hosts {
            let country_code = country.as_str().to_lowercase();
            match host.alias_range_upper_bound {
                Some(upper_bound) => {
                    for alias_index in 1..=upper_bound {
                        let host_uri = format!("{}.{}", alias_index, host.base);

                        proxy_hosts
                            .entry(country_code.clone())
                            .and_modify(|h| h.push(host_uri.clone()))
                            .or_insert(vec![host_uri]);
                    }
                }
                None => {
                    proxy_hosts.insert(country_code.clone(), vec![host.base.to_owned()]);
                }
            }
        }

        self.proxy_hosts = Some(proxy_hosts);
        self
    }

    pub fn create_request(&self, method: &str, bucket: &str, object: &str) -> SignedRequest {
        let uri = format!("/{bucket}/{object}", bucket = bucket, object = object);
        SignedRequest::new(method, "s3", &self.region, &uri)
    }

    fn get_proxy_hosts(&self, country: Option<String>) -> Option<&Vec<String>> {
        country.and_then(|c| self.proxy_hosts.as_ref()?.get(&c))
    }

    pub fn sign_request(&self, req: &mut SignedRequest, country: Option<String>) -> Result<String> {
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
        country: Option<String>,
        method: &str,
        bucket: &str,
        object: &str,
    ) -> Result<String> {
        self.sign_request(&mut self.create_request(method, bucket, object), country)
    }
}

#[cfg(test)]
mod tests {
    use crate::{app::util::ProxyHost, s3::Client};
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn set_proxy_hosts_test() {
        let mut client = Client::new(
            "key",
            "secret",
            "region",
            "endpoint",
            ::std::time::Duration::from_secs(300),
        );

        let mut hosts = HashMap::new();
        let ua_host = ProxyHost {
            base: "ua.example.org".to_string(),
            alias_range_upper_bound: Some(2),
        };
        hosts.insert("ua".to_string(), ua_host);

        let es_host = ProxyHost {
            base: "es.example.org".to_string(),
            alias_range_upper_bound: None,
        };
        hosts.insert("es".to_string(), es_host);

        let result = client.set_proxy_hosts(&hosts);

        let mut expected = BTreeMap::new();
        expected.insert(
            "ua".to_string(),
            vec![
                "1.ua.example.org".to_string(),
                "2.ua.example.org".to_string(),
            ],
        );
        expected.insert("es".to_string(), vec!["es.example.org".to_string()]);

        assert_eq!(result.proxy_hosts, Some(expected));
    }
}
