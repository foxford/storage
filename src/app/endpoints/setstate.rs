use axum::extract::{Path, State};
use http::Response;
use std::sync::Arc;

use svc_utils::extractors::AccountIdExtractor;

use crate::app::context::AppContext;

pub async fn read(
    State(ctx): State<Arc<AppContext>>,
    AccountIdExtractor(sub): AccountIdExtractor,
    Path(set): Path<String>,
    Path(object): Path<String>,
) -> Response<String> {
    unimplemented!();
}

pub async fn backend_read(
    State(ctx): State<Arc<AppContext>>,
    AccountIdExtractor(sub): AccountIdExtractor,
    Path(back): Path<String>,
    Path(set): Path<String>,
    Path(object): Path<String>,
) -> Response<String> {
    unimplemented!();
}
