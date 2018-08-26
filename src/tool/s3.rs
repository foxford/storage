use rusoto_core::credential::AwsCredentials;
use rusoto_core::signature::SignedRequest;
use rusoto_core::Region;
use std::time::Duration;

#[derive(Debug)]
pub struct Options {
    credentials: AwsCredentials,
    region: Region,
    expires_in: Duration,
}

impl Options {
    pub fn new(
        key: &str,
        secret: &str,
        region: &str,
        endpoint: &str,
        expires_in: Duration,
    ) -> Options {
        let region = Region::Custom {
            name: region.to_string(),
            endpoint: endpoint.to_string(),
        };
        let credentials = AwsCredentials::new(key, secret, None, None);

        Options {
            credentials,
            region,
            expires_in,
        }
    }

    pub fn presigned_url(self: &Options, method: &str, bucket: &str, key: &str) -> String {
        let uri = format!("/{bucket}/{key}", bucket = bucket, key = key);
        let mut req = SignedRequest::new(method, "s3", &self.region, &uri);
        req.generate_presigned_url(&self.credentials, &self.expires_in)
    }
}
