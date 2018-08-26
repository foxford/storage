pub mod object;
pub mod set;

use tool;
use warp;
use warp::http::{Response, StatusCode};

#[derive(Debug)]
pub struct State {
    pub s3: tool::s3::Options,
}

pub type StateRef = ::std::sync::Arc<State>;

pub fn redirect(uri: &str) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("location", uri)
        .body(""))
}
