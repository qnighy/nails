use futures::prelude::*;

use std::sync::Arc;

use contextful::Context;
use futures::task::Poll;
use hyper::client::service::Service as HyperService;
use hyper::{Body, Method, Request, Response, StatusCode};

use crate::error::NailsError;
use crate::request::Preroute;
use crate::routing::{Routable, Router};

#[derive(Debug)]
pub struct ServiceWithContext<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    pub service: Service<Ctx>,
    pub ctx: Ctx,
}

impl<Ctx> Clone for ServiceWithContext<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            ctx: self.ctx.clone(),
        }
    }
}

impl<'a, Ctx, HyperCtx> HyperService<&'a HyperCtx> for ServiceWithContext<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    type Response = ServiceWithContext<Ctx>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn call(&mut self, _ctx: &HyperCtx) -> Self::Future {
        future::ok(self.clone())
    }

    // TODO: implement poll_ready for better connection throttling
    fn poll_ready(
        &mut self,
        _cx: &mut futures::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl<'a, Ctx> HyperService<Request<Body>> for ServiceWithContext<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let inner = self.service.inner.clone();
        let ctx = self.ctx.clone();
        async move { inner.respond(&ctx, req).await }.boxed()
    }

    // TODO: implement poll_ready for better connection throttling
    fn poll_ready(
        &mut self,
        _cx: &mut futures::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
pub struct Service<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    inner: Arc<ServiceInner<Ctx>>,
}

impl<Ctx> Service<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    pub fn builder() -> Builder<Ctx> {
        Builder::new()
    }

    pub fn with_context(self, ctx: &Ctx) -> ServiceWithContext<Ctx> {
        ServiceWithContext {
            service: self,
            ctx: ctx.clone(),
        }
    }
}

impl<Ctx> Clone for Service<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Builder<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    inner: Option<ServiceInner<Ctx>>,
}

impl<Ctx: Context> Builder<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: Some(ServiceInner {
                router: Router::new(),
            }),
        }
    }

    pub fn finish(&mut self) -> Service<Ctx> {
        let inner = self.inner.take().expect("Builder::finish called twice");
        Service {
            inner: Arc::new(inner),
        }
    }

    fn inner_mut(&mut self) -> &mut ServiceInner<Ctx> {
        self.inner
            .as_mut()
            .expect("this builder is already finished")
    }

    pub fn add_route<R>(&mut self, route: R) -> &mut Self
    where
        R: Routable<Ctx = Ctx> + Send + Sync + 'static,
    {
        self.inner_mut().router.add_route(route);
        self
    }

    pub fn add_function_route<F, Fut, Req>(&mut self, route: F) -> &mut Self
    where
        F: Fn(Ctx, Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<Body>, NailsError>> + Send + 'static,
        Req: Preroute + Send + 'static,
    {
        self.inner_mut().router.add_function_route(route);
        self
    }
}

#[derive(Debug)]
struct ServiceInner<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    router: Router<Ctx>,
}

impl<Ctx> ServiceInner<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    async fn respond(
        &self,
        ctx: &Ctx,
        req: Request<Body>,
    ) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
        if req.method() == Method::OPTIONS && req.headers().get("Origin").is_some() {
            // CORS hack.
            // TODO: move this out to middleware
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "*")
                .header("Access-Control-Allow-Headers", "*")
                .body(Body::empty())
                .unwrap());
        }
        let resp = if self.router.match_path(req.method(), req.uri().path()) {
            match self.router.respond(ctx, req).await {
                Ok(resp) => resp,
                Err(e) => e.to_response(),
            }
        } else {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
        };
        let resp = {
            let mut resp = resp;
            // CORS hack.
            // TODO: move this out to middleware
            resp.headers_mut()
                .append("Access-Control-Allow-Origin", "*".parse().unwrap());
            resp
        };
        Ok(resp)
    }
}
