use super::{redirect, StateRef};

use warp;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

pub fn route(state: StateRef) -> BoxedFilter<(impl Reply,)> {
    let path = path!("buckets" / String / "objects" / String).and(warp::index());
    let state = warp::any().map(move || state.clone());
    let head_index = warp::head().and(path).and(state.clone()).and_then(head);
    let read_index = warp::get2().and(path).and(state.clone()).and_then(read);
    head_index.or(read_index).boxed()
}

pub fn head(bucket: String, key: String, state: StateRef) -> Result<impl Reply, Rejection> {
    redirect(&state.s3.presigned_url("HEAD", &bucket, &key))
}

pub fn read(bucket: String, key: String, state: StateRef) -> Result<impl Reply, Rejection> {
    redirect(&state.s3.presigned_url("GET", &bucket, &key))
}
