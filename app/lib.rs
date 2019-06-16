#![feature(async_await)]

use futures::compat::{Compat, Future01CompatExt};
use futures::TryStreamExt;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};
use runtime::net::TcpListener;

static TEXT: &str = "Hello, World!";

pub async fn server() -> failure::Fallible<()> {
    let new_svc = || service_fn_ok(|_req| Response::new(Body::from(TEXT)));

    let mut listener = TcpListener::bind("127.0.0.1:3000")?;
    println!("Listening on {}", listener.local_addr()?);

    let incoming = listener.incoming().map_ok(Compat::new).compat();

    let server = Server::builder(incoming)
        .executor(Compat::new(Runtime))
        .serve(new_svc)
        .compat();

    server.await?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Runtime;

impl futures::task::Spawn for &Runtime {
    fn spawn_obj(
        &mut self,
        future: futures::future::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        let _handle = runtime::spawn(future);
        Ok(())
    }
}
