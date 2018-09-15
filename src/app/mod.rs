mod authn;
mod config;

use http;
use tool;
use tower_web::middleware::cors::CorsBuilder;
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
        #[get("/api/v1/buckets/:bucket/objects/:object")]
        fn read(&self, bucket: String, object: String/*, _sub: Option<authn::Subject>*/) -> Result<http::Response<&'static str>, ()> {
            redirect(&self.s3.presigned_url("GET", &bucket, &object))
        }
    }

    impl Set {
        #[get("/api/v1/buckets/:bucket/sets/:set/objects/:object")]
        fn read(&self, bucket: String, set: String, object: String/*, _sub: Option<authn::Subject>*/) -> Result<http::Response<&'static str>, ()> {
            redirect(&self.s3.presigned_url("GET", &bucket, &Self::s3_object(&set, &object)))
        }

        fn s3_object(set: &str, object: &str) -> String {
            format!("{set}.{object}", set = set, object = object)
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
    use http::{header, Method};
    use std::collections::HashSet;

    let config = config::load().expect("Failed to load config");
    info!("App config: {:?}", config);

    let allow_headers: HashSet<header::HeaderName> = [
        header::CACHE_CONTROL,
        header::IF_MATCH,
        header::IF_MODIFIED_SINCE,
        header::IF_NONE_MATCH,
        header::IF_UNMODIFIED_SINCE,
        header::RANGE,
    ]
        .iter()
        .cloned()
        .collect();

    let cors = CorsBuilder::new()
        .allow_origins(config.cors.allow_origins)
        .allow_methods(vec![Method::GET])
        .allow_headers(allow_headers)
        .max_age(config.cors.max_age)
        .build();

    let addr = "0.0.0.0:8080".parse().expect("Invalid address");
    info!("Listening on http://{}", addr);

    let s3 = S3ClientRef::new(s3);
    ServiceBuilder::new()
        .config(config.authn)
        .resource(Object { s3: s3.clone() })
        .resource(Set { s3: s3.clone() })
        .middleware(cors)
        .run(&addr)
        .unwrap();
}
