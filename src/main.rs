extern crate config;
extern crate env_logger;
extern crate jsonwebtoken as jose;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate serde;
#[macro_use]
extern crate tower_web;
extern crate failure;
extern crate http;

mod app;
mod tool;

fn main() {
    env_logger::init();

    use std::env::var;
    let key = var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be specified");
    let secret = var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be specified");
    let endpoint = var("AWS_ENDPOINT").expect("AWS_ENDPOINT must be specified");
    let region = var("AWS_REGION").expect("AWS_REGION must be specified");

    let s3 = tool::s3::Client::new(
        &key,
        &secret,
        &region,
        &endpoint,
        ::std::time::Duration::from_secs(300),
    );

    app::run(s3);
}
