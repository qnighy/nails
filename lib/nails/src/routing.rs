use futures::prelude::*;

use std::fmt;
use std::marker::PhantomData;
use std::pin::Pin;

use hyper::{Body, Method, Request, Response};

use crate::request::FromRequest;

pub struct Router {
    routes: Vec<Box<dyn Routable + Send + 'static>>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route<R>(&mut self, route: R)
    where
        R: Routable + Send + 'static,
    {
        self.routes.push(Box::new(route));
    }

    pub fn add_function_route<F, Fut, Req>(&mut self, route: F)
    where
        F: Fn(Req) -> Fut + Send + 'static,
        Fut: Future<Output = Response<Body>> + Send + 'static,
        Req: FromRequest + 'static,
    {
        self.add_route(FunctionRoute::new(route))
    }
}

impl fmt::Debug for Router {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Router").finish()
    }
}

impl Routable for Router {
    fn match_path(&self, method: &Method, path: &str) -> bool {
        self.routes
            .iter()
            .any(|route| route.match_path(method, path))
    }
    fn respond(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'static>> {
        let method = req.method();
        let path = req.uri().path();
        let mut matched_route = None;
        for route in &self.routes {
            if route.match_path(method, path) {
                if matched_route.is_some() {
                    // TODO: make this Err
                    panic!("multiple matching routes");
                }
                matched_route = Some(route);
            }
        }
        matched_route.expect("no route matched").respond(req)
    }
}

pub trait Routable {
    fn path_prefix_hint(&self) -> &str {
        ""
    }
    fn match_path(&self, method: &Method, path: &str) -> bool;
    // TODO: Result
    fn respond(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'static>>;
}

pub struct FunctionRoute<F, Req> {
    f: F,
    _marker: PhantomData<fn(Req)>,
}

impl<F, Fut, Req> FunctionRoute<F, Req>
where
    F: Fn(Req) -> Fut,
    Fut: Future<Output = Response<Body>> + Send + 'static,
    Req: FromRequest,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}

impl<F, Fut, Req> Routable for FunctionRoute<F, Req>
where
    F: Fn(Req) -> Fut,
    Fut: Future<Output = Response<Body>> + Send + 'static,
    Req: FromRequest,
{
    fn path_prefix_hint(&self) -> &str {
        Req::path_prefix_hint()
    }

    fn match_path(&self, method: &Method, path: &str) -> bool {
        Req::match_path(method, path)
    }

    fn respond(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'static>> {
        let req = FromRequest::from_request(req);
        (self.f)(req).boxed()
    }
}
