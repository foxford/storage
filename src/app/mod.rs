mod authn;
mod authz;
mod config;
mod util;

use failure;
use http;
use std::collections::BTreeMap;
use tool;
use tower_web::error::{Error, ErrorKind};

type S3ClientRef = ::std::sync::Arc<tool::s3::Client>;

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
    authz: config::AuthzMap,
    ns: config::Namespaces,
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

impl_web! {

    impl Object {
        #[get("/api/v1/buckets/:bucket/objects/:object")]
        fn read(&self, bucket: String, object: String/*, _sub: Option<authn::Subject>*/) -> Result<http::Response<&'static str>, ()> {
            // TODO: Add authorization
            redirect(&self.s3.presigned_url("GET", &bucket, &object))
        }
    }

    impl Set {
        #[get("/api/v1/buckets/:bucket/sets/:set/objects/:object")]
        fn read(&self, bucket: String, set: String, object: String/*, _sub: Option<authn::Subject>*/) -> Result<http::Response<&'static str>, ()> {
            // TODO: Add authorization
            redirect(&self.s3.presigned_url("GET", &bucket, &s3_object(&set, &object)))
        }
    }

    impl Sign {
        #[post("/api/v1/sign")]
        #[content_type("json")]
        fn sign_route(&self, body: SignPayload, subject: authn::Subject) -> Result<Result<SignResponse, Error>, ()> {
            Ok(self.sign(body, subject))
        }

        fn sign(&self, body: SignPayload, subject: authn::Subject) -> Result<SignResponse, Error> {
            // TODO: return 400 – unimplemented action
            let authz_action = action(&body.method).map_err(|_err| Error::from(ErrorKind::bad_request()))?;
            let (s3_object, authz_object) = match body.set {
                Some(ref set) => (
                    s3_object(&set, &body.object),
                    authz::Entity::new(&self.ns.app, vec!["buckets", &body.bucket, "sets", set]),
                ),
                None => (
                    body.object.to_owned(),
                    authz::Entity::new(&self.ns.app, vec!["buckets", &body.bucket, "objects", &body.object]),
                )
            };

            // NOTE: authorize only "update" and "delete" actions
            match authz_action {
                "update" | "delete" => {
                    let authz_subject = authz::Entity::new(&subject.audience, vec!["accounts", &subject.id]);

                    // TODO: return 403 - access forbidden
                    let authz = self.authz.get(&subject.audience).ok_or_else(|| {
                        error!("Authz: no configuration for {} audience", subject.audience);
                        Error::from(ErrorKind::forbidden())
                    })?;
                    let authz_req = authz::Request::new(&authz_subject, &authz_object, authz_action);
                    (config::Authz::client(authz)).authorize(&authz_req).map_err(|err| {
                        error!("Authz: {}, {:?}", err, authz_req);
                        Error::from(ErrorKind::forbidden())
                    })?;
                }
                _ => ()
            };

            let mut builder = util::S3SignedRequestBuilder::new()
                .method(&body.method)
                .bucket(&body.bucket)
                .object(&s3_object);
            for (key, val) in body.headers {
                builder = builder.add_header(&key, &val);
            }

            // TODO: return 422 - S3 client fails to build a signed URI
            let uri = builder.build(&self.s3).map_err(|err| {
                error!("S3Client: {}", err);
                Error::from(ErrorKind::unprocessable_entity())
            })?;
            Ok(SignResponse { uri })
        }
    }

}

fn action(method: &str) -> Result<&str, failure::Error> {
    match method {
        "HEAD" => Ok("read"),
        "GET" => Ok("read"),
        "PUT" => Ok("update"),
        "DELETE" => Ok("delete"),
        _ => Err(failure::err_msg("bad method")),
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

pub(crate) fn run(s3: tool::s3::Client) {
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
        authz: config.authz,
        ns: config.namespaces,
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
