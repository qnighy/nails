use hyper::{Body, Response};
use nails::response::ErrorResponse;
use nails::FromRequest;
use serde::Serialize;

use crate::context::AppCtx;

#[derive(Debug, FromRequest)]
#[nails(path = "/api/tags")]
pub(crate) struct ListTagsRequest;

#[derive(Debug, Serialize)]
pub(crate) struct ListTagsResponseBody {
    tags: Vec<String>,
}

pub(crate) async fn list_tags(
    _ctx: AppCtx,
    _req: ListTagsRequest,
) -> Result<Response<Body>, ErrorResponse> {
    let body = ListTagsResponseBody {
        tags: vec![String::from("tag1"), String::from("tag2")],
    };
    Ok(super::json_response(&body))
}
