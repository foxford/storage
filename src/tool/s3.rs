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

    pub(crate) fn create_request(&self, method: &str, bucket: &str, object: &str) -> SignedRequest {
        let uri = format!("/{bucket}/{object}", bucket = bucket, object = object);
        SignedRequest::new(method, "s3", &self.region, &uri)
    }

    pub(crate) fn sign_request(&self, req: &mut SignedRequest) -> String {
        req.generate_presigned_url(&self.credentials, &self.expires_in)
    }

    pub(crate) fn presigned_url(self: &Self, method: &str, bucket: &str, object: &str) -> String {
        self.sign_request(&mut self.create_request(method, bucket, object))
    }
}
