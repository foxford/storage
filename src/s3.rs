use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Context, Result};
use rusoto_core::credential::AwsCredentials;
use rusoto_core::signature::SignedRequest;
use rusoto_core::Region;
use url::Url;

#[derive(Debug)]
pub(crate) struct Client {
    credentials: AwsCredentials,
    region: Region,
    expires_in: Duration,
    proxy_host: Option<Vec<String>>,
    counter: AtomicUsize,
}

impl Client {
    pub(crate) fn new(
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
            proxy_host: None,
            counter: AtomicUsize::new(0),
        }
    }

    pub(crate) fn set_proxy_hosts(&mut self, host: &[String]) -> &mut Self {
        self.proxy_host = Some(host.to_owned());
        self
    }

    pub(crate) fn create_request(&self, method: &str, bucket: &str, object: &str) -> SignedRequest {
        let uri = format!("/{bucket}/{object}", bucket = bucket, object = object);
        SignedRequest::new(method, "s3", &self.region, &uri)
    }

    pub(crate) fn sign_request(&self, req: &mut SignedRequest) -> Result<String> {
        let url = req.generate_presigned_url(&self.credentials, &self.expires_in, false);

        if let Some(ref proxy_hosts) = self.proxy_host {
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

    pub(crate) fn presigned_url(&self, method: &str, bucket: &str, object: &str) -> Result<String> {
        self.sign_request(&mut self.create_request(method, bucket, object))
    }
}
