#![feature(async_await)]

use futures::compat::{Compat, Future01CompatExt};
use futures::{FutureExt, TryFutureExt, TryStreamExt};
use hyper::service::service_fn;
use hyper::Server;
use runtime::net::TcpListener;

mod routes;

#[runtime::main]
async fn main() -> failure::Fallible<()> {
    server().await?;
    Ok(())
}

pub async fn server() -> failure::Fallible<()> {
    let new_svc = || service_fn(|req| crate::routes::route(req).boxed().compat());

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
