#![feature(async_await)]

#[macro_use]
extern crate diesel;

use structopt::StructOpt;

use futures::compat::{Compat, Future01CompatExt};
use futures::TryStreamExt;
use hyper::Server;
use runtime::net::TcpListener;

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

    let mut listener = TcpListener::bind(("127.0.0.1", opt.port.unwrap_or(3000)))?;
    println!("Listening on {}", listener.local_addr()?);

    let incoming = listener.incoming().map_ok(Compat::new).compat();

    let server = Server::builder(incoming)
        .executor(Compat::new(runtime::task::Spawner::new()))
        .serve(svc.with_context(ctx))
        .compat();

    server.await?;

    Ok(())
}
