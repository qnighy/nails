use hyper::{Body, Method, Request, Response, StatusCode};

use nails::{FromRequest, Routable, Router};

pub(crate) async fn route(req: Request<Body>) -> failure::Fallible<Response<Body>> {
    let router = {
        let mut router = Router::new();
        router.add_function_route(index);
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
