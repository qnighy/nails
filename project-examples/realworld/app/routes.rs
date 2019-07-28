use hyper::{Body, Response};
use serde::Serialize;

use nails::response::ErrorResponse;
use nails::{FromRequest, Service};

use crate::context::AppCtx;

mod articles;
mod posts;
mod tags;

pub fn build_route(ctx: &AppCtx) -> Service<AppCtx> {
    Service::builder()
        .add_function_route(index)
        .add_function_route(posts::get_post)
        .add_function_route(tags::list_tags)
        .add_function_route(articles::list_articles)
        .finish(ctx)
}

#[derive(Debug, FromRequest)]
#[nails(path = "/")]
struct IndexRequest {
    #[nails(query)]
    a: Vec<String>,
}

async fn index(_ctx: AppCtx, req: IndexRequest) -> Result<Response<Body>, ErrorResponse> {
    Ok(Response::new(Body::from(format!(
        "Hello, world! {:?}",
        req.a
    ))))
}

fn json_response<T: Serialize>(body: &T) -> Response<Body> {
    Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}
