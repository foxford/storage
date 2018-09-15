use rusoto_core::credential::AwsCredentials;
use rusoto_core::signature::SignedRequest;
use rusoto_core::Region;
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct Client {
    credentials: AwsCredentials,
    region: Region,
    expires_in: Duration,
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
        }
    }

    pub(crate) fn presigned_url(self: &Client, method: &str, bucket: &str, object: &str) -> String {
        let uri = format!("/{bucket}/{object}", bucket = bucket, object = object);
        let mut req = SignedRequest::new(method, "s3", &self.region, &uri);
        req.generate_presigned_url(&self.credentials, &self.expires_in)
    }
}
