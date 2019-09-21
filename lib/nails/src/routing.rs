use futures::prelude::*;

use std::fmt;
use std::marker::PhantomData;

use async_trait::async_trait;
use contextful::Context;
use hyper::{Body, Method, Request, Response};

use crate::error::NailsError;
use crate::request::Preroute;

pub struct Router<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    routes: Vec<Box<dyn Routable<Ctx = Ctx> + Send + Sync + 'static>>,
    _marker: PhantomData<fn(Ctx)>,
}

impl<Ctx> Router<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn add_route<R>(&mut self, route: R)
    where
        R: Routable<Ctx = Ctx> + Send + Sync + 'static,
    {
        self.routes.push(Box::new(route));
    }

    pub fn add_function_route<F, Fut, Req>(&mut self, route: F)
    where
        F: Fn(Ctx, Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<Body>, NailsError>> + Send + 'static,
        Req: Preroute + Send + 'static,
    {
        self.add_route(FunctionRoute::new(route))
    }
}

impl<Ctx> fmt::Debug for Router<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Router").finish()
    }
}

#[async_trait]
impl<Ctx> Routable for Router<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    type Ctx = Ctx;

    fn match_path(&self, method: &Method, path: &str) -> bool {
        self.routes
            .iter()
            .any(|route| route.match_path(method, path))
    }
    async fn respond(
        &self,
        ctx: &Self::Ctx,
        req: Request<Body>,
    ) -> Result<Response<Body>, NailsError> {
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
        matched_route
            .expect("no route matched")
            .respond(&ctx, req)
            .await
    }
}

#[async_trait]
pub trait Routable {
    type Ctx: Context + Send + Sync + 'static;

    fn path_prefix_hint(&self) -> &str {
        ""
    }
    fn match_path(&self, method: &Method, path: &str) -> bool;
    // TODO: Result
    async fn respond(
        &self,
        ctx: &Self::Ctx,
        req: Request<Body>,
    ) -> Result<Response<Body>, NailsError>;
}

pub struct FunctionRoute<Ctx, F, Req> {
    f: F,
    _marker: PhantomData<fn(Ctx, Req)>,
}

impl<Ctx, F, Fut, Req> FunctionRoute<Ctx, F, Req>
where
    Ctx: Context + Send + Sync + 'static,
    F: Fn(Ctx, Req) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Response<Body>, NailsError>> + Send + 'static,
    Req: Preroute + Send,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<Ctx, F, Fut, Req> Routable for FunctionRoute<Ctx, F, Req>
where
    Ctx: Context + Send + Sync + 'static,
    F: Fn(Ctx, Req) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Response<Body>, NailsError>> + Send + 'static,
    Req: Preroute + Send,
{
    type Ctx = Ctx;

    fn path_prefix_hint(&self) -> &str {
        Req::path_prefix_hint()
    }

    fn match_path(&self, method: &Method, path: &str) -> bool {
        Req::match_path(method, path)
    }

    async fn respond(
        &self,
        ctx: &Self::Ctx,
        req: Request<Body>,
    ) -> Result<Response<Body>, NailsError> {
        let req = Preroute::from_request(req).await?;
        (self.f)(ctx.clone(), req).await
    }
}
