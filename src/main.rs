#![feature(async_await)]

#[runtime::main]
async fn main() -> failure::Fallible<()> {
    sayho::server().await?;
    Ok(())
}
