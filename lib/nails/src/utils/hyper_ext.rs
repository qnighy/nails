//! Utilities for using `hyper` with `runtime`.

use futures::compat::Compat;
use hyper::Server;
use runtime::net::{TcpListener, TcpStream};
use runtime::task::Spawner;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct AddrIncoming {
    inner: TcpListener,
}

impl futures01::Stream for AddrIncoming {
    type Item = Compat<TcpStream>;
    type Error = std::io::Error;

    fn poll(&mut self) -> futures01::Poll<Option<Self::Item>, Self::Error> {
        use futures01::Async::*;

        match Compat::new(self.inner.incoming()).poll() {
            Ok(Ready(Some(stream))) => Ok(Ready(Some(Compat::new(stream)))),
            Ok(Ready(None)) => Ok(Ready(None)),
            Ok(NotReady) => Ok(NotReady),
            Err(e) => Err(e),
        }
    }
}

pub trait ServerBindExt {
    type Builder;

    /// Binds to the provided address, and returns a [`Builder`](Builder).
    ///
    /// # Panics
    ///
    /// This method will panic if binding to the address fails.
    fn bind2(addr: &SocketAddr) -> Self::Builder;
}

impl ServerBindExt for Server<AddrIncoming, ()> {
    type Builder = hyper::server::Builder<AddrIncoming, Compat<Spawner>>;

    fn bind2(addr: &SocketAddr) -> Self::Builder {
        let incoming = TcpListener::bind(addr).unwrap_or_else(|e| {
            panic!("error binding to {}: {}", addr, e);
        });
        Server::builder(AddrIncoming { inner: incoming }).executor(Compat::new(Spawner::new()))
    }
}
