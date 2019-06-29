use hyper::{Body, Method, Request, Response, StatusCode};

use nails::{FromRequest, Routable, Router};

pub(crate) async fn route(req: Request<Body>) -> failure::Fallible<Response<Body>> {
    let router = {
        let mut router = Router::new();
        router.add_function_route(index);
        router
    };

    let resp = if router.match_path(req.method(), req.uri().path()) {
        router.respond(req)
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
    };
    Ok(resp)
}

#[derive(Debug)]
struct IndexRequest {}

impl FromRequest for IndexRequest {
    fn path_prefix_hint() -> &'static str {
        "/"
    }
    fn match_path(method: &Method, path: &str) -> bool {
        (*method == Method::GET || *method == Method::POST) && path == "/"
    }

    fn from_request(_req: Request<Body>) -> Self {
        IndexRequest {}
    }
}

fn index(_req: IndexRequest) -> Response<Body> {
    Response::new(Body::from("Hello, world!"))
}
