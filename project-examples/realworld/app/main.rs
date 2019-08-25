#[macro_use]
extern crate diesel;

use structopt::StructOpt;

use futures::compat::Future01CompatExt;
use hyper::Server;
use nails::utils::hyper_ext::ServerBindExt;

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
    let addr = (host, port).into();

    let server = Server::bind2(&addr).serve(svc.with_context(ctx)).compat();
    println!("Listening on {}", addr);

    server.await?;

    Ok(())
}
