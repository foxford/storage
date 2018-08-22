extern crate env_logger;
extern crate log;
#[macro_use]
extern crate warp;
extern crate rusoto_core;
extern crate rusoto_s3;

use rusoto_core::credential::AwsCredentials;
use rusoto_core::signature::SignedRequest;
use rusoto_core::Region;
use std::env;
use std::sync::Arc;
use warp::http::{Response, StatusCode};
use warp::Filter;

#[derive(Debug)]
struct S3Options {
    region: Region,
    credentials: AwsCredentials,
    expires_in: ::std::time::Duration,
}

#[derive(Debug)]
struct State {
    s3: S3Options,
}
type StateRef = Arc<State>;

fn main() {
    env_logger::init();

    let key = env::var("AWS_ACCESS_KEY_ID").unwrap();
    let secret = env::var("AWS_SECRET_ACCESS_KEY").unwrap();
    let endpoint = env::var("AWS_ENDPOINT").unwrap();
    let region = env::var("AWS_REGION").unwrap();

    let region = Region::Custom {
        name: region,
        endpoint,
    };
    let credentials = AwsCredentials::new(key, secret, None, None);
    let s3 = S3Options {
        region,
        credentials,
        expires_in: ::std::time::Duration::from_secs(300),
    };

    let state = Arc::new(State { s3 });
    let state = warp::any().map(move || state.clone());

    let api = path!("api" / "v1");
    let bucket = path!("buckets" / String);
    let object = path!("objects" / String);
    let set = path!("sets" / String);

    let object_path = bucket.and(object).and(warp::index()).and(state.clone());
    let object_head_index = warp::head().and(object_path.clone()).and_then(head_object);
    let object_get_index = warp::get2().and(object_path.clone()).and_then(read_object);
    let object_index = object_head_index.or(object_get_index);

    let set_path = bucket
        .and(set)
        .and(object)
        .and(warp::index())
        .and(state.clone());
    let set_head_index = warp::head().and(set_path.clone()).and_then(head_set);
    let set_get_index = warp::get2().and(set_path.clone()).and_then(read_set);
    let set_index = set_head_index.or(set_get_index);

    let route = api.and(object_index.or(set_index));

    warp::serve(route).run(([0, 0, 0, 0], 8080));
}

fn head_object(
    bucket: String,
    key: String,
    state: StateRef,
) -> Result<impl warp::Reply, warp::Rejection> {
    redirect(&presigned_url("HEAD", &bucket, &key, &state.s3))
}

fn read_object(
    bucket: String,
    key: String,
    state: StateRef,
) -> Result<impl warp::Reply, warp::Rejection> {
    redirect(&presigned_url("GET", &bucket, &key, &state.s3))
}

fn head_set(
    bucket: String,
    set: String,
    key: String,
    state: StateRef,
) -> Result<impl warp::Reply, warp::Rejection> {
    let s3_key = format!("{}.{}", set, key);
    redirect(&presigned_url("HEAD", &bucket, &s3_key, &state.s3))
}

fn read_set(
    bucket: String,
    set: String,
    key: String,
    state: StateRef,
) -> Result<impl warp::Reply, warp::Rejection> {
    let s3_key = format!("{}.{}", set, key);
    redirect(&presigned_url("GET", &bucket, &s3_key, &state.s3))
}

fn redirect(url: &str) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("location", url)
        .body(""))
}

fn presigned_url(method: &str, bucket: &str, key: &str, options: &S3Options) -> String {
    let uri = format!("/{bucket}/{key}", bucket = bucket, key = key);
    let mut req = SignedRequest::new(method, "s3", &options.region, &uri);
    req.generate_presigned_url(&options.credentials, &options.expires_in)
}
