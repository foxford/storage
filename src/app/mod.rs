use crate::authn::AccountId;
use crate::authz;
use crate::s3;
use failure::format_err;
use http::{self, StatusCode};
use log::{error, info};
use std::collections::BTreeMap;
use tower_web::Error;

type S3ClientRef = ::std::sync::Arc<s3::Client>;

#[derive(Debug)]
struct Object {
    s3: S3ClientRef,
}

#[derive(Debug)]
struct Set {
    s3: S3ClientRef,
}

#[derive(Debug)]
struct Sign {
    application_id: AccountId,
    authz: authz::ConfigMap,
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

#[derive(Debug, Deserialize)]
pub(crate) struct Cors {
    #[serde(deserialize_with = "crate::serde::allowed_origins")]
    #[serde(default)]
    pub(crate) allow_origins: tower_web::middleware::cors::AllowedOrigins,
    #[serde(deserialize_with = "crate::serde::duration")]
    #[serde(default)]
    pub(crate) max_age: std::time::Duration,
}

impl_web! {

    impl Object {
        #[get("/api/v1/buckets/:bucket/objects/:object")]
        fn read(&self, bucket: String, object: String/*, _sub: Option<AccountId>*/) -> Result<http::Response<&'static str>, ()> {
            // TODO: Add authorization
            redirect(&self.s3.presigned_url("GET", &bucket, &object))
        }
    }

    impl Set {
        #[get("/api/v1/buckets/:bucket/sets/:set/objects/:object")]
        fn read(&self, bucket: String, set: String, object: String/*, _sub: Option<AccountId>*/) -> Result<http::Response<&'static str>, ()> {
            // TODO: Add authorization
            redirect(&self.s3.presigned_url("GET", &bucket, &s3_object(&set, &object)))
        }
    }

    impl Sign {
        #[post("/api/v1/sign")]
        #[content_type("json")]
        fn sign_route(&self, body: SignPayload, sub: AccountId) -> Result<Result<SignResponse, Error>, ()> {
            // TODO: improve error logging
            Ok(self.sign(body, sub).map_err(|err| { error!("{}", err); err }))
        }

        fn sign(&self, body: SignPayload, sub: AccountId) -> Result<SignResponse, Error> {
            let error = || Error::builder().kind("sign_error", "Error signing a request");

            let object = {
                let ns = self.application_id.to_string();
                let (object, zobj) = match body.set {
                    Some(ref set) => (
                        s3_object(&set, &body.object),
                        authz::Entity::new(&ns, vec!["buckets", &body.bucket, "sets", set]),
                    ),
                    None => (
                        body.object.to_owned(),
                        authz::Entity::new(&ns, vec!["buckets", &body.bucket, "objects", &body.object]),
                    )
                };
                let zact = parse_action(&body.method)
                    .map_err(|err| error().status(StatusCode::BAD_REQUEST).detail(&err.to_string()).build())?;

                // NOTE: authorize only "update" and "delete" actions
                match zact {
                    "update" | "delete" => {
                        let zsub = authz::Entity::new(sub.audience(), vec!["accounts", sub.label()]);
                        let authz = self.authz.get(sub.audience()).ok_or_else(|| {
                            let detail = format!("no authz configuration for the audience = {}", sub.audience());
                            error().status(StatusCode::FORBIDDEN).detail(&detail).build()
                        })?;
                        let zreq = authz::Request::new(&zsub, &zobj, zact);
                        (authz::Config::client(authz)).authorize(&zreq)?;
                    }
                    _ => ()
                };

                object
            };

            let mut builder = util::S3SignedRequestBuilder::new()
                .method(&body.method)
                .bucket(&body.bucket)
                .object(&object);
            for (key, val) in body.headers {
                builder = builder.add_header(&key, &val);
            }
            let uri = builder.build(&self.s3)?;

            Ok(SignResponse { uri })
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

fn redirect(uri: &str) -> Result<http::Response<&'static str>, ()> {
    use http::{Response, StatusCode};

    Ok(Response::builder()
        .header("location", uri)
        .status(StatusCode::SEE_OTHER)
        .body("")
        .unwrap())
}

pub(crate) fn run(s3: s3::Client) {
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
        .allow_origins(config.cors.allow_origins)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(allow_headers)
        .allow_credentials(true)
        .max_age(config.cors.max_age)
        .build();

    let log = LogMiddleware::new("storage::web");

    // Resources
    let s3 = S3ClientRef::new(s3);

    let object = Object { s3: s3.clone() };
    let set = Set { s3: s3.clone() };
    let sign = Sign {
        application_id: config.id,
        authz: config.authz,
        s3: s3.clone(),
    };

    let addr = "0.0.0.0:8080".parse().expect("Invalid address");
    info!("Listening on http://{}", addr);

    ServiceBuilder::new()
        .config(config.authn)
        .resource(object)
        .resource(set)
        .resource(sign)
        .middleware(log)
        .middleware(cors)
        .run(&addr)
        .unwrap();
}

mod config;
mod util;
