use futures::prelude::*;

pub use crate::error::ErrorResponse;
pub use crate::request::{parse_query, FromBody, FromPath, FromQuery, Preroute};
pub use futures::future::BoxFuture;
pub use hyper::{Body, Method, Request};

pub fn box_future<'a, T: Future + Send + 'a>(x: T) -> BoxFuture<'a, T::Output> {
    x.boxed()
}
