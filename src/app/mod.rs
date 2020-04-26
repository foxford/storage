use failure::format_err;
use futures::{future, Future};
use http::{Response, StatusCode};
use log::{error, info};
use std::collections::BTreeMap;
use std::sync::Arc;
use svc_authn::AccountId;
use svc_authz::cache::Cache;
use tower_web::Error;

use crate::s3;
use util::Subject;

////////////////////////////////////////////////////////////////////////////////

type S3ClientRef = Arc<util::S3Clients>;

#[derive(Debug)]
struct Object {
    authz: svc_authz::ClientMap,
    authz_wo: bool,
    aud_estm: Arc<util::AudienceEstimator>,
    s3: S3ClientRef,
}

#[derive(Debug)]
struct Set {
    authz: svc_authz::ClientMap,
    authz_wo: bool,
    aud_estm: Arc<util::AudienceEstimator>,
    s3: S3ClientRef,
}

#[derive(Debug)]
struct Sign {
    application_id: AccountId,
    authz: svc_authz::ClientMap,
    aud_estm: Arc<util::AudienceEstimator>,
    s3: S3ClientRef,
}

#[derive(Debug, Extract)]
struct SignPayload {
    bucket: String,
    set: Option<String>,
    object: String,
    method: String,
    headers: BTreeMap<String, String>,
}

#[derive(Response)]
#[web(status = "200")]
struct SignResponse {
    uri: String,
}

#[derive(Debug)]
struct Healthz {}

#[derive(Debug, Deserialize)]
pub(crate) struct HttpConfig {
    listener_address: String,
    cors: Cors,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Cors {
    #[serde(deserialize_with = "crate::serde::allowed_origins")]
    #[serde(default)]
    pub(crate) allow_origins: tower_web::middleware::cors::AllowedOrigins,
    #[serde(deserialize_with = "crate::serde::duration")]
    #[serde(default)]
    pub(crate) max_age: std::time::Duration,
}

////////////////////////////////////////////////////////////////////////////////

impl_web! {

    impl Object {
        #[get("/api/v1/buckets/:bucket/objects/:object")]
        fn read(&self, bucket: String, object: String, maybe_sub: Option<Subject>) -> impl Future<Item = Result<Response<&'static str>, Error>, Error = ()> {
            self.read_ns(String::from(crate::app::util::S3_DEFAULT_CLIENT), bucket, object, maybe_sub)
        }

        #[get("/api/v1/backends/:back/buckets/:bucket/objects/:object")]
        fn read_ns(&self, back: String, bucket: String, object: String, maybe_sub: Option<Subject>) -> impl Future<Item = Result<Response<&'static str>, Error>, Error = ()> {
            let error = || Error::builder().kind("object_error", "Error reading an object using Object API");
            let wrap_error = |err| { error!("{}", err); future::ok(Err(err)) };
            let s3 = self.s3.clone();
            let s3 = match s3.get(&back) {
                Some(val) => val.clone(),
                None => return future::Either::B(wrap_error(error().status(StatusCode::NOT_FOUND).detail(&format!("Backend '{}' is not found", &back)).build()))
            };

            match self.authz_wo {
                false => future::Either::A(match maybe_sub {
                    Some(sub) => future::Either::A(self.read_authz(&s3, bucket, object, sub)),
                    None => future::Either::B(wrap_error(error().status(StatusCode::FORBIDDEN).detail("missing an access token").build()))
                }),
                // NOTE: ignore an authorization step if the 'AUTHZ_WRITE_ONLY' environment variable is set
                true => future::Either::B(future::ok(s3
                    .presigned_url("GET", &bucket, &object)
                    .map(|ref uri| redirect(uri))
                    .map_err(|err| error()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .detail(&err.to_string())
                        .build())))
            }
        }

        fn read_authz(&self, s3: &s3::Client, bucket: String, object: String, sub: Subject) -> impl Future<Item = Result<Response<&'static str>, Error>, Error = ()> {
            let error = || Error::builder().kind("object_error", "Error reading an object using Object API");
            let wrap_error = |err| { error!("{}", err); future::ok(Err(err)) };

            let zobj = vec!["buckets", &bucket, "objects", &object];
            let zact = "read";

            let resp = s3
                .presigned_url("GET", &bucket, &object)
                .map(|ref uri| redirect(uri))
                .map_err(|err| error()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .detail(&err.to_string())
                    .build());

            match self.aud_estm.estimate(&bucket) {
                Err(err) => future::Either::A(wrap_error(err)),
                Ok(audience) => future::Either::B(self
                    .authz
                    .authorize(audience, &sub, zobj, zact)
                    .and_then(move |zresp| match zresp {
                        Err(err) => future::Either::A(wrap_error(error().status(StatusCode::FORBIDDEN).detail(&err.to_string()).build())),
                        Ok(_) => future::Either::B(future::ok(resp)),
                    })),
            }
        }
    }

    impl Set {
        #[get("/api/v1/buckets/:bucket/sets/:set/objects/:object")]
        fn read(&self, bucket: String, set: String, object: String, maybe_sub: Option<Subject>) -> impl Future<Item = Result<Response<&'static str>, Error>, Error = ()> {
            self.read_ns(String::from(crate::app::util::S3_DEFAULT_CLIENT), bucket, set, object, maybe_sub)
        }

        #[get("/api/v1/backends/:back/buckets/:bucket/sets/:set/objects/:object")]
        fn read_ns(&self, back: String, bucket: String, set: String, object: String, maybe_sub: Option<Subject>) -> impl Future<Item = Result<Response<&'static str>, Error>, Error = ()> {
            let error = || Error::builder().kind("set_error", "Error reading an object using Set API");
            let wrap_error = |err| { error!("{}", err); future::ok(Err(err)) };
            let s3 = self.s3.clone();
            let s3 = match s3.get(&back) {
                Some(val) => val.clone(),
                None => return future::Either::B(wrap_error(error().status(StatusCode::NOT_FOUND).detail(&format!("Backend '{}' is not found", &back)).build()))
            };

            match self.authz_wo {
                false => future::Either::A(match maybe_sub {
                    Some(sub) => future::Either::A(self.read_authz(&s3, bucket, set, object, sub)),
                    None => future::Either::B(wrap_error(error().status(StatusCode::FORBIDDEN).detail("missing an access token").build()))
                }),
                // NOTE: ignore an authorization step if the 'AUTHZ_WRITE_ONLY' environment variable is set
                true => future::Either::B(future::ok(s3
                    .presigned_url("GET", &bucket, &s3_object(&set, &object))
                    .map(|ref uri| redirect(uri))
                    .map_err(|err| error()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .detail(&err.to_string())
                        .build()))),
            }
        }

        fn read_authz(&self, s3: &s3::Client, bucket: String, set: String, object: String, sub: Subject) -> impl Future<Item = Result<Response<&'static str>, Error>, Error = ()> {
            let error = || Error::builder().kind("set_error", "Error reading an object using Set API");
            let wrap_error = |err| { error!("{}", err); future::ok(Err(err)) };

            let zobj = vec!["buckets", &bucket, "sets", &set];
            let zact = "read";

            let resp = s3
                .presigned_url("GET", &bucket, &s3_object(&set, &object))
                .map(|ref uri| redirect(uri))
                .map_err(|err| error()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .detail(&err.to_string())
                    .build());

            match self.aud_estm.estimate(&bucket) {
                Err(err) => future::Either::A(wrap_error(err)),
                Ok(audience) => future::Either::B(self
                    .authz
                    .authorize(audience, &sub, zobj, zact)
                    .and_then(move |zresp| match zresp {
                        Err(err) => future::Either::A(wrap_error(error().status(StatusCode::FORBIDDEN).detail(&err.to_string()).build())),
                        Ok(_) => future::Either::B(future::ok(resp)),
                    })),
            }
        }
    }

    impl Sign {
        #[post("/api/v1/sign")]
        #[content_type("json")]
        fn sign(&self, body: SignPayload, sub: Subject) -> impl Future<Item = Result<SignResponse, Error>, Error = ()> {
            self.sign_ns(String::from(crate::app::util::S3_DEFAULT_CLIENT), body, sub)
        }

        #[post("/api/v1/backends/:back/sign")]
        #[content_type("json")]
        fn sign_ns(&self, back: String, body: SignPayload, sub: Subject) -> impl Future<Item = Result<SignResponse, Error>, Error = ()> {
            let error = || Error::builder().kind("sign_error", "Error signing a request");
            let wrap_error = |err| { error!("{}", err); future::ok(Err(err)) };
            let s3 = self.s3.clone();
            let s3 = match s3.get(&back) {
                Some(val) => val.clone(),
                None => return future::Either::A(wrap_error(error().status(StatusCode::NOT_FOUND).detail(&format!("Backend '{}' is not found", &back)).build()))
            };

            // Authz subject, object, and action
            let (object, zobj) = match body.set {
                Some(ref set) => (
                    s3_object(&set, &body.object),
                    vec!["buckets", &body.bucket, "sets", set]
                ),
                None => (
                    body.object.to_owned(),
                    vec!["buckets", &body.bucket, "objects", &body.object]
                )
            };
            let zact = match parse_action(&body.method) {
                Ok(val) => val,
                Err(err) => return future::Either::A(wrap_error(error().status(StatusCode::FORBIDDEN).detail(&err.to_string()).build()))
            };

            // URI builder
            let mut builder = util::S3SignedRequestBuilder::new()
                .method(&body.method)
                .bucket(&body.bucket)
                .object(&object);

            for (key, val) in body.headers {
                builder = builder.add_header(&key, &val);
            }

            let uri = match builder.build(&s3) {
                Ok(val) => val,
                Err(err) => return future::Either::A(wrap_error(err))
            };

            match self.aud_estm.estimate(&body.bucket) {
                Ok(audience) => {
                    future::Either::B(self.authz.authorize(audience, &sub, zobj, zact).and_then(move |zresp| match zresp {
                        Err(err) => future::Either::A(wrap_error(error().status(StatusCode::FORBIDDEN).detail(&err.to_string()).build())),
                        Ok(_) => future::Either::B(future::ok(Ok(SignResponse { uri }))),
                    }))
                },
                Err(err) => {
                    future::Either::A(wrap_error(err))
                }
            }
        }
    }

    impl Healthz {
        #[get("/healthz")]
        fn healthz(&self) -> Result<Response<&'static str>, ()> {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body("")
                .unwrap())
        }
    }

}

fn parse_action(method: &str) -> Result<&str, failure::Error> {
    match method {
        "HEAD" => Ok("read"),
        "GET" => Ok("read"),
        "PUT" => Ok("update"),
        "DELETE" => Ok("delete"),
        _ => Err(format_err!("invalid method = {}", method)),
    }
}

fn s3_object(set: &str, object: &str) -> String {
    format!("{set}.{object}", set = set, object = object)
}

fn redirect(uri: &str) -> Response<&'static str> {
    Response::builder()
        .header("location", uri)
        .status(StatusCode::SEE_OTHER)
        .body("")
        .unwrap()
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn run(cache: Option<Cache>, authz_wo: bool) {
    use http::{header, Method};
    use std::collections::HashSet;
    use tower_web::middleware::cors::CorsBuilder;
    use tower_web::middleware::log::LogMiddleware;
    use tower_web::ServiceBuilder;

    // Config
    let config = config::load().expect("Failed to load config");
    info!("App config: {:?}", config);

    // Middleware
    let allow_headers: HashSet<header::HeaderName> = [
        header::AUTHORIZATION,
        header::CACHE_CONTROL,
        header::CONTENT_LENGTH,
        header::CONTENT_TYPE,
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
        .allow_origins(config.http.cors.allow_origins)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(allow_headers)
        .allow_credentials(true)
        .max_age(config.http.cors.max_age)
        .build();

    let log = LogMiddleware::new("storage::http");

    // Resources
    let s3_clients =
        util::read_s3_config(config.backend.as_ref()).expect("Error reading s3 config");

    let s3 = S3ClientRef::new(s3_clients);

    // Authz
    let aud_estm = Arc::new(util::AudienceEstimator::new(&config.authz));
    let authz = svc_authz::ClientMap::new(&config.id, cache, config.authz)
        .expect("Error converting authz config to clients");

    let object = Object {
        authz: authz.clone(),
        authz_wo,
        aud_estm: aud_estm.clone(),
        s3: s3.clone(),
    };
    let set = Set {
        authz: authz.clone(),
        authz_wo,
        aud_estm: aud_estm.clone(),
        s3: s3.clone(),
    };
    let sign = Sign {
        application_id: config.id,
        authz,
        aud_estm,
        s3,
    };
    let healthz = Healthz {};

    let addr = config
        .http
        .listener_address
        .parse()
        .expect("Error parsing HTTP listener address");
    ServiceBuilder::new()
        .config(config.authn)
        .resource(object)
        .resource(set)
        .resource(sign)
        .resource(healthz)
        .middleware(log)
        .middleware(cors)
        .run(&addr)
        .expect("Error running the HTTP listener");
}

////////////////////////////////////////////////////////////////////////////////

mod config;
pub(crate) mod util;
