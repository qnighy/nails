use futures::prelude::*;

use std::sync::Arc;

use contextful::Context;
use futures::compat::Compat;
use hyper::service::{MakeService, Service as HyperService};
use hyper::{Body, Request, Response, StatusCode};

use crate::request::FromRequest;
use crate::response::ErrorResponse;
use crate::routing::{Routable, Router};

#[derive(Debug)]
pub struct Service<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    inner: Arc<ServiceInner<Ctx>>,
    ctx: Ctx,
}

impl<Ctx> Service<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    pub fn builder() -> Builder<Ctx> {
        Builder::new()
    }
}

impl<Ctx> Clone for Service<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            ctx: self.ctx.clone(),
        }
    }
}

impl<'a, Ctx, HyperCtx> MakeService<HyperCtx> for Service<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Service = Service<Ctx>;
    type Future = futures01::future::FutureResult<Self::Service, Self::MakeError>;
    type MakeError = Box<dyn std::error::Error + Send + Sync>;

    fn make_service(&mut self, _ctx: HyperCtx) -> Self::Future {
        futures01::future::ok(self.clone())
    }

    // TODO: implement poll_ready for better connection throttling
}

impl<'a, Ctx> HyperService for Service<Ctx>
where
    Ctx: Context + Send + Sync + 'static,
{
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Compat<future::BoxFuture<'static, Result<Response<Self::ResBody>, Self::Error>>>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let inner = self.inner.clone();
        let ctx = self.ctx.clone();
        async move { inner.respond(&ctx, req).await }
            .boxed()
            .compat()
    }

    // TODO: implement poll_ready for better connection throttling
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

    pub fn finish(&mut self, ctx: &Ctx) -> Service<Ctx> {
        let inner = self.inner.take().expect("Builder::finish called twice");
        Service {
            inner: Arc::new(inner),
            ctx: ctx.clone(),
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
        F: Fn(Req) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<Body>, ErrorResponse>> + Send + 'static,
        Req: FromRequest + 'static,
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
