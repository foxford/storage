use http;
use tool;
use tower_web::ServiceBuilder;

type S3ClientRef = ::std::sync::Arc<tool::s3::Client>;

#[derive(Debug)]
struct Object {
    s3: S3ClientRef,
}

#[derive(Debug)]
struct Set {
    s3: S3ClientRef,
}

impl_web! {

    impl Object {
        #[get("/api/v1/buckets/:bucket/objects/:key")]
        fn read(&self, bucket: String, key: String) -> Result<http::Response<&'static str>, ()> {
            redirect(&self.s3.presigned_url("GET", &bucket, &key))
        }
    }

    impl Set {
        #[get("/api/v1/buckets/:bucket/sets/:set/objects/:key")]
        #[content_type("json")]
        fn read(&self, bucket: String, set: String, key: String) -> Result<http::Response<&'static str>, ()> {
            redirect(&self.s3.presigned_url("GET", &bucket, &Self::s3_key(&set, &key)))
        }

        fn s3_key(set: &str, key: &str) -> String {
            format!("{set}.{key}", set = set, key = key)
        }
    }

}

fn redirect(uri: &str) -> Result<http::Response<&'static str>, ()> {
    use http::{Response, StatusCode};

    Ok(Response::builder()
        .header("location", uri)
        .status(StatusCode::SEE_OTHER)
        .body("")
        .unwrap())
}

pub(crate) fn run(s3: tool::s3::Client) {
    let addr = "0.0.0.0:8080".parse().expect("Invalid address");
    info!("Listening on http://{}", addr);

    let s3 = S3ClientRef::new(s3);
    ServiceBuilder::new()
        .resource(Object { s3: s3.clone() })
        .resource(Set { s3: s3.clone() })
        .run(&addr)
        .unwrap();
}
