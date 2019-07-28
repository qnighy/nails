use diesel::prelude::*;
use hyper::{Body, Response};
use nails::response::ErrorResponse;
use nails::FromRequest;
use serde::Serialize;

use crate::context::AppCtx;
use crate::models::Tag;

#[derive(Debug, FromRequest)]
#[nails(path = "/api/tags")]
pub(crate) struct ListTagsRequest;

#[derive(Debug, Serialize)]
pub(crate) struct ListTagsResponseBody {
    tags: Vec<String>,
}

pub(crate) async fn list_tags(
    ctx: AppCtx,
    _req: ListTagsRequest,
) -> Result<Response<Body>, ErrorResponse> {
    use crate::schema::tags::dsl::*;

    // TODO: async
    let conn = ctx.db.get().unwrap(); // TODO: handle errors
    let all_tags = tags.load::<Tag>(&conn).unwrap(); // TODO: handle errors
    let body = ListTagsResponseBody {
        tags: all_tags.iter().map(|t| t.tag.clone()).collect(),
    };
    Ok(super::json_response(&body))
}
