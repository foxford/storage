extern crate env_logger;
extern crate log;
#[macro_use]
extern crate warp;
extern crate rusoto_core;
extern crate rusoto_s3;

use std::env;
use warp::Filter;

pub mod http;
pub mod tool;

fn main() {
    env_logger::init();

    let key = env::var("AWS_ACCESS_KEY_ID").unwrap();
    let secret = env::var("AWS_SECRET_ACCESS_KEY").unwrap();
    let endpoint = env::var("AWS_ENDPOINT").unwrap();
    let region = env::var("AWS_REGION").unwrap();

    let state = http::StateRef::new(http::State {
        s3: tool::s3::Options::new(
            &key,
            &secret,
            &region,
            &endpoint,
            ::std::time::Duration::from_secs(300),
        ),
    });
    let route = path!("api" / "v1")
        .and(http::object::route(state.clone()).or(http::set::route(state.clone())));

    warp::serve(route).run(([0, 0, 0, 0], 8080));
}
