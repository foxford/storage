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

    pub fn set_proxy_hosts(&mut self, proxy_hosts: &HashMap<String, Vec<ProxyHost>>) -> &mut Self {
        let mut resulting_hosts: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for (country, hosts) in proxy_hosts {
            for host in hosts {
                let mut _hosts = host
                    .alias_range_upper_bound
                    .map(|upper_bound| {
                        (1..=upper_bound)
                            .map(|idx| format!("{idx}.{}", host.base))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or(vec![host.base.to_owned()]);

                resulting_hosts
                    .entry(country.as_str().to_lowercase())
                    .and_modify(|h| h.append(_hosts.as_mut()))
                    .or_insert(_hosts);
            }
        }

        self.proxy_hosts = Some(resulting_hosts);
        self
    }

    pub fn create_request(&self, method: &str, bucket: &str, object: &str) -> SignedRequest {
        let uri = format!("/{bucket}/{object}", bucket = bucket, object = object);
        SignedRequest::new(method, "s3", &self.region, &uri)
    }

    fn get_proxy_hosts(&self, country: Option<String>) -> Option<&Vec<String>> {
        country.and_then(|c| self.proxy_hosts.as_ref()?.get(&c.to_lowercase()))
    }

    pub fn sign_request(&self, req: &mut SignedRequest, country: Option<String>) -> Result<String> {
        let url = req.generate_presigned_url(&self.credentials, &self.expires_in, false);

        if let Some(proxy_hosts) = self.get_proxy_hosts(country) {
            let mut parsed_url = Url::parse(&url).context("failed to parse generated uri")?;
            let idx = self.counter.fetch_add(1, Ordering::Acquire) % proxy_hosts.len();
            let proxy_host = proxy_hosts.get(idx).map(|h| h.as_str());

            parsed_url
                .set_host(proxy_host)
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
        let ua_hosts = vec![
            ProxyHost {
                base: "ua1.example.org".to_string(),
                alias_range_upper_bound: None,
            },
            ProxyHost {
                base: "ua2.example.org".to_string(),
                alias_range_upper_bound: Some(2),
            },
        ];
        hosts.insert("ua".to_string(), ua_hosts);

        let es_host = ProxyHost {
            base: "es.example.org".to_string(),
            alias_range_upper_bound: None,
        };
        hosts.insert("es".to_string(), vec![es_host]);

        let result = client.set_proxy_hosts(&hosts);

        let mut expected = BTreeMap::new();
        expected.insert(
            "ua".to_string(),
            vec![
                "ua1.example.org".to_string(),
                "1.ua2.example.org".to_string(),
                "2.ua2.example.org".to_string(),
            ],
        );
        expected.insert("es".to_string(), vec!["es.example.org".to_string()]);

        assert_eq!(result.proxy_hosts, Some(expected));
    }
}
