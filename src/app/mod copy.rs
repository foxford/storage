use anyhow::format_err;
use futures::{future, Future};
use http::{Response, StatusCode};
use log::{error, info};
use std::collections::BTreeMap;
use std::string::ToString;
use std::sync::Arc;
use svc_authn::AccountId;
use svc_authz::cache::RedisCache;
use tower_web::Error;
use futures_util::future::try_future::TryFutureExt

use self::config::AudienceSettings;
use crate::db::ConnectionPool;
use util::Subject;

////////////////////////////////////////////////////////////////////////////////

const MAX_LIMIT: i64 = 25;

////////////////////////////////////////////////////////////////////////////////

type S3ClientRef = ::std::sync::Arc<util::S3Clients>;


#[derive(Debug)]
struct SetState {
    authz: svc_authz::ClientMap,
    aud_estm: Arc<util::AudienceEstimator>,
    s3: S3ClientRef,
    audiences_settings: BTreeMap<String, AudienceSettings>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct SignState {
    application_id: AccountId,
    authz: svc_authz::ClientMap,
    aud_estm: Arc<util::AudienceEstimator>,
    s3: S3ClientRef,
    audiences_settings: BTreeMap<String, AudienceSettings>,
}

#[derive(Debug, Extract)]
struct SignPayload {
    set: String,
    object: String,
    method: String,
    headers: BTreeMap<String, String>,
}

#[derive(Response)]
#[web(status = "200")]
struct SignResponse {
    uri: String,
}

#[derive(Response)]
struct RedirectResponse {
}

#[derive(Debug)]
struct Healthz {}

impl_web! {
    impl SetState {
        #[get("/api/v2/sets/:set/objects/:object")]
        fn read(&self, set: String, object: String, sub: Subject, referer: Option<String>) -> impl Future<Item = Result<Response<String>, Error>, Error = ()> {
            self.read_ns(String::from(crate::app::util::S3_DEFAULT_CLIENT), set, object, sub, referer)
        }

        #[get("/api/v2/backends/:back/sets/:set/objects/:object")]
        fn read_ns(&self, back: String, set: String, object: String, sub: Subject, referer: Option<String>) -> impl Future<Item = Result<Response<String>, Error>, Error = ()> {
            let error = || Error::builder().kind("set_read_error", "Error reading an object by set");

            let zobj = vec!["sets", &set];
            let zact = "read";
            let s3 = self.s3.clone();
            let s3 = match s3.get(&back) {
                Some(val) => val.clone(),
                None => return future::Either::A(wrap_error(error().status(StatusCode::NOT_FOUND).detail(&format!("Backend '{}' is not found", &back)).build()))
            };

            match self.aud_estm.parse_set(&set) {
                Ok(set_s) => {
                    if let Err(e) = self.valid_referer(&set_s.bucket().to_string(), referer) {
                        return future::Either::A(wrap_error(e));
                    }

                    future::Either::B(self
                        .authz
                        .authorize(set_s.bucket().audience().to_string(), sub, zobj, zact.to_string())
                        .await
                        .and_then(move |zresp| match zresp {
                            Err(err) => future::Either::A(wrap_error(error().status(StatusCode::FORBIDDEN).detail(&err.to_string()).build())),
                            Ok(_) => {
                                let bucket = set_s.bucket().to_string();
                                let object = s3_object(set_s.label(), &object);

                                future::Either::B(future::ok(s3
                                    .presigned_url("GET", &bucket, &object)
                                    .map(|ref uri| redirect(uri))
                                    .map_err(|err| error()
                                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                                        .detail(&err.to_string())
                                        .build())))
                        }}))
                },
                Err(err) => {
                    future::Either::A(wrap_error(err))
                }
            }
        }

        fn valid_referer(&self, bucket: &str, referer: Option<String>) -> Result<(), Error> {
            let error = || Error::builder().kind("set_read_error", "Error reading an object using Set API");

            match self.aud_estm.estimate(bucket) {
                Ok(aud) => match self.audiences_settings.get(aud) {
                    Some(aud_settings) => if !aud_settings.valid_referer(referer.as_deref()) {
                        let e = error().status(StatusCode::FORBIDDEN).detail("Invalid request").build();
                        return Err(e);
                    }
                    None => {
                        let e = error().status(StatusCode::NOT_FOUND).detail(&format!("Audience settings for bucket '{}' not found", &bucket)).build();
                        return Err(e);
                    }
                }
                Err(err) => {
                    let e = error().status(StatusCode::NOT_FOUND).detail(&format!("Audience estimate for bucket '{}' not found, err = {}", &bucket, err)).build();
                    return Err(e);
                }
            }

            Ok(())
        }
    }

    impl SignState {
        #[post("/api/v2/sign")]
        #[content_type("json")]
        fn sign(&self, body: SignPayload, sub: Subject, referer: Option<String>) -> impl Future<Item = Result<SignResponse, Error>, Error = ()> {
            self.sign_ns(String::from(crate::app::util::S3_DEFAULT_CLIENT), body, sub, referer)
        }

        #[post("/api/v2/backends/:back/sign")]
        #[content_type("json")]
        fn sign_ns(&self, back: String, body: SignPayload, sub: Subject, referer: Option<String>) -> impl Future<Item = Result<SignResponse, Error>, Error = ()> {
            let error = || Error::builder().kind("sign_error", "Error signing a request");

            if let Ok(set_s) = self.aud_estm.parse_set(&body.set) {
                if let Err(e) = self.valid_referer(&set_s.bucket().to_string(), referer) {
                    return future::Either::A(wrap_error(e));
                }
            }

            let zobj = vec!["sets", &body.set];
            let zact = match parse_action(&body.method) {
                Ok(val) => val,
                Err(err) => return future::Either::A(wrap_error(error().status(StatusCode::FORBIDDEN).detail(&err.to_string()).build()))
            };
            let s3 = self.s3.clone();
            let s3 = match s3.get(&back) {
                Some(val) => val.clone(),
                None => return future::Either::A(wrap_error(error().status(StatusCode::NOT_FOUND).detail(&format!("Backend '{}' is not found", &back)).build()))
            };

            match self.aud_estm.parse_set(&body.set) {
                Ok(set_s) => {
                    future::Either::B(self.authz.authorize(set_s.bucket().audience().to_string(), sub, zobj, zact.to_string()).await.and_then(move |zresp| match zresp {
                        Err(err) => future::Either::A(wrap_error(error().status(StatusCode::FORBIDDEN).detail(&err.to_string()).build())),
                        Ok(_) => {
                            // URI builder
                            let mut builder = util::S3SignedRequestBuilder::new()
                                .method(&body.method)
                                .bucket(&set_s.bucket().to_string())
                                .object(&s3_object(set_s.label(), &body.object));
                            for (key, val) in body.headers {
                                builder = builder.add_header(&key, &val);
                            }

                            future::Either::B(future::ok(builder.build(&s3).map(|uri| SignResponse { uri })))
                    }}))
                },
                Err(err) => future::Either::A(wrap_error(err))
            }
        }

        fn valid_referer(&self, bucket: &str, referer: Option<String>) -> Result<(), Error> {
            let error = || Error::builder().kind("sign_error", "Error signing a request");

            match self.aud_estm.estimate(bucket) {
                Ok(aud) => match self.audiences_settings.get(aud) {
                    Some(aud_settings) => if !aud_settings.valid_referer(referer.as_deref()) {
                        let e = error().status(StatusCode::FORBIDDEN).detail("Invalid request").build();
                        return Err(e);
                    }
                    None => {
                        let e = error().status(StatusCode::NOT_FOUND).detail(&format!("Audience settings for bucket '{}' not found", &bucket)).build();
                        return Err(e);
                    }
                }
                Err(err) => {
                    let e = error().status(StatusCode::NOT_FOUND).detail(&format!("Audience estimate for bucket '{}' not found, err = {}", &bucket, err)).build();
                    return Err(e);
                }
            }

            Ok(())
        }
    }

    impl Healthz {
        #[get("/healthz")]
        fn healthz(&self) -> Result<&'static str, ()> {
            Ok("")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpConfig {
    listener_address: String,
    cors: Cors,
}

#[derive(Debug, Deserialize)]
pub struct Cors {
    #[serde(deserialize_with = "crate::serde::allowed_origins")]
    #[serde(default)]
    pub allow_origins: tower_web::middleware::cors::AllowedOrigins,
    #[serde(deserialize_with = "crate::serde::duration")]
    #[serde(default)]
    pub max_age: std::time::Duration,
}

////////////////////////////////////////////////////////////////////////////////

fn parse_action(method: &str) -> anyhow::Result<&str> {
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
        .header("Timing-Allow-Origin", "*")
        .status(StatusCode::SEE_OTHER)
        .body("")
        .unwrap()
}

fn wrap_error<T>(err: Error) -> impl Future<Item = Result<T, Error>, Error = ()> {
    error!("{}", err);
    future::ok(Err(err))
}

////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::mutable_key_type)]
pub fn run(db: Option<ConnectionPool>, cache: Option<RedisCache>) {
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
        header::HeaderName::from_static("x-request-type"),
    ]
    .iter()
    .cloned()
    .collect();

    let cors = CorsBuilder::new()
        .allow_origins(config.http.cors.allow_origins.clone())
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
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
    let authz = svc_authz::ClientMap::new(&config.id, cache, config.authz.clone())
        .expect("Error converting authz config to clients");

    let object = ObjectState {
        authz: authz.clone(),
        aud_estm: aud_estm.clone(),
        s3: s3.clone(),
        audiences_settings: config.audiences_settings.clone(),
    };
    let set = SetState {
        authz: authz.clone(),
        aud_estm: aud_estm.clone(),
        s3: s3.clone(),
        audiences_settings: config.audiences_settings.clone(),
    };
    let sign = SignState {
        application_id: config.id.clone(),
        authz: authz.clone(),
        aud_estm: aud_estm.clone(),
        s3: s3.clone(),
        audiences_settings: config.audiences_settings.clone(),
    };
    let tag = TagState {
        authz,
        aud_estm,
        s3,
        db,
    };
    let healthz = Healthz {};

    let addr = config
        .http
        .listener_address
        .parse()
        .expect("Error parsing HTTP listener address");
    ServiceBuilder::new()
        .config(config)
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
pub mod util;
