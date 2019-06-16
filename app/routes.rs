use hyper::{Body, Request, Response, StatusCode};

pub(crate) async fn route(req: Request<Body>) -> failure::Fallible<Response<Body>> {
    let path = req.uri().path();
    let resp = if path == "/" {
        index(req).await?
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(""))
            .unwrap()
    };
    Ok(resp)
}

async fn index(_req: Request<Body>) -> failure::Fallible<Response<Body>> {
    Ok(Response::new(Body::from("Hello, world!")))
}
