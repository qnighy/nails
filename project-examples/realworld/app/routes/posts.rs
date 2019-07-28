use hyper::{Body, Response};
use nails::response::ErrorResponse;
use nails::FromRequest;
use serde::Serialize;

use crate::context::AppCtx;

#[derive(Debug, FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub(crate) struct GetPostRequest {
    id: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct GetPostBody {
    post: Post,
}

#[derive(Debug, Serialize)]
pub(crate) struct Post {
    body: String,
}

pub(crate) async fn get_post(
    _ctx: AppCtx,
    _req: GetPostRequest,
) -> Result<Response<Body>, ErrorResponse> {
    let body = GetPostBody {
        post: Post {
            body: String::from("foo"),
        },
    };
    Ok(super::json_response(&body))
}
