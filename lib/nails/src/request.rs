use hyper::{Body, Method, Request};

pub use nails_derive::FromRequest;

pub trait FromRequest: Sized {
    fn path_prefix_hint() -> &'static str {
        ""
    }
    fn match_path(method: &Method, path: &str) -> bool;

    // TODO: Result<Self>
    // TODO: Request<Body> -> RoutableRequest
    fn from_request(req: Request<Body>) -> Self;
}
