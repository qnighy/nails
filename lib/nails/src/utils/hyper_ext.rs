//! Utilities for using `hyper` with `runtime`.

use futures::prelude::*;

use super::tokio02_ext::Compat as Tokio02Compat;
use futures::task::Poll;
use hyper::Server;
use runtime::net::{TcpListener, TcpStream};
use runtime::task::Spawner;
use std::net::SocketAddr;
use std::pin::Pin;

#[derive(Debug)]
pub struct AddrIncoming {
    inner: TcpListener,
}

impl Stream for AddrIncoming {
    type Item = std::io::Result<Tokio02Compat<TcpStream>>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut futures::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner.incoming())
            .poll_next(cx)
            .map(|x| x.map(|x| x.map(Tokio02Compat)))
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

    fn bind2_mut(addr: &mut SocketAddr) -> Self::Builder;
}

impl ServerBindExt for Server<AddrIncoming, ()> {
    type Builder = hyper::server::Builder<AddrIncoming, Tokio02Compat<Spawner>>;

    fn bind2(addr: &SocketAddr) -> Self::Builder {
        let incoming = TcpListener::bind(addr).unwrap_or_else(|e| {
            panic!("error binding to {}: {}", addr, e);
        });
        Server::builder(AddrIncoming { inner: incoming })
            .executor(Tokio02Compat::new(Spawner::new()))
    }

    fn bind2_mut(addr: &mut SocketAddr) -> Self::Builder {
        let incoming = TcpListener::bind(&*addr).unwrap_or_else(|e| {
            panic!("error binding to {}: {}", addr, e);
        });
        if let Ok(l_addr) = incoming.local_addr() {
            *addr = l_addr;
        }
        Server::builder(AddrIncoming { inner: incoming })
            .executor(Tokio02Compat::new(Spawner::new()))
    }
}
