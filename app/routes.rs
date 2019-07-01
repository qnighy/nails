use hyper::{Body, Method, Request, Response, StatusCode};
use serde::Serialize;
use serde_derive::Serialize;

use nails::{FromRequest, Routable, Router};

pub(crate) async fn route(req: Request<Body>) -> failure::Fallible<Response<Body>> {
    let router = {
        let mut router = Router::new();
        router.add_function_route(index);
        router.add_function_route(get_post);
        router
    };

    let resp = if router.match_path(req.method(), req.uri().path()) {
        router.respond(req).await
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
    };
    Ok(resp)
}

#[derive(Debug, FromRequest)]
#[nails(path = "/")]
struct IndexRequest {
    #[nails(query)]
    a: Vec<String>,
}

async fn index(req: IndexRequest) -> Response<Body> {
    Response::new(Body::from(format!("Hello, world! {:?}", req.a)))
}

#[derive(Debug, FromRequest)]
#[nails(path = "/api/posts/1")]
struct GetPostRequest {}

#[derive(Debug, Serialize)]
struct GetPostBody {
    post: Post,
}

#[derive(Debug, Serialize)]
struct Post {
    body: String,
}

async fn get_post(_req: GetPostRequest) -> Response<Body> {
    let body = GetPostBody {
        post: Post {
            body: String::from("foo"),
        }
    };
    Response::new(Body::from(serde_json::to_string(&body).unwrap()))
}
