#![feature(async_await)]

#[runtime::main]
async fn main() {
    sayho::server().await;
}
