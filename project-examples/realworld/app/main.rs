#[macro_use]
extern crate diesel;

use hyper::Server;
use nails::utils::hyper_ext::ServerBindExt;
use structopt::StructOpt;

use crate::context::AppCtx;

mod context;
mod models;
mod routes;
mod schema;

#[derive(Debug, Clone, StructOpt)]
struct CommandOpt {
    #[structopt(subcommand)]
    subcommand: SubcommandOpt,
}

#[derive(Debug, Clone, StructOpt)]
enum SubcommandOpt {
    #[structopt(name = "server")]
    ServerCommandOpt(ServerCommandOpt),
}

#[runtime::main]
async fn main() -> failure::Fallible<()> {
    dotenv::dotenv().ok();

    let opt = CommandOpt::from_args();
    let ctx = AppCtx::new();
    match opt.subcommand {
        SubcommandOpt::ServerCommandOpt(ref server_opt) => {
            server(&ctx, server_opt).await?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, StructOpt)]
pub(crate) struct ServerCommandOpt {
    #[structopt(short = "p", help = "on which port to listen")]
    port: Option<u16>,
}

pub(crate) async fn server(ctx: &AppCtx, opt: &ServerCommandOpt) -> failure::Fallible<()> {
    let svc = crate::routes::build_route(ctx);

    let host: std::net::IpAddr = "127.0.0.1".parse().unwrap();
    let port = opt.port.unwrap_or(3000);
    let mut addr = (host, port).into();

    let server = Server::bind2_mut(&mut addr).serve(svc.with_context(ctx));
    println!("Listening on {}", addr);

    server.await?;

    Ok(())
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct TestServer {
    addr: std::net::SocketAddr,
    shutdown: Option<futures::channel::oneshot::Sender<()>>,
}

#[cfg(test)]
impl TestServer {
    pub(crate) fn addr(&self) -> std::net::SocketAddr {
        self.addr
    }
}

#[cfg(test)]
impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown.send(()).ok();
        }
    }
}

#[cfg(test)]
async fn server_for_test(ctx: &AppCtx) -> failure::Fallible<TestServer> {
    use futures::channel::oneshot;
    use futures::prelude::*;

    let svc = crate::routes::build_route(ctx);

    let host: std::net::IpAddr = "127.0.0.1".parse().unwrap();
    let port = 0;
    let mut addr = (host, port).into();

    let server = Server::bind2_mut(&mut addr).serve(svc.with_context(ctx));

    let (tx, rx) = oneshot::channel();

    runtime::spawn(async move {
        match server.with_graceful_shutdown(rx.map(|_| ())).await {
            Ok(()) => {}
            Err(e) => {
                // TODO: log it properly
                eprintln!("Server error: {}", e);
            }
        }
    });

    Ok(TestServer {
        addr,
        shutdown: Some(tx),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use surf::http::StatusCode;

    #[runtime::test]
    async fn test_server() {
        let server = init_test().await;

        let url: String = format!("http://{}/", server.addr());
        let mut res = surf::get(url).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.body_string().await.unwrap();
        assert_eq!(body, "Hello, world! []");
        eprintln!("res = {:?}", res);
    }

    async fn init_test() -> TestServer {
        dotenv::dotenv().ok();
        let ctx = AppCtx::new();
        server_for_test(&ctx).await.unwrap()
    }
}
