use futures::prelude::*;

use futures::task::{Context, Poll, Spawn, SpawnError, SpawnExt};
use std::io;
use std::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Compat<T>(pub T);

impl<T> Compat<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
    pub fn into_inner(self) -> T {
        self.0
    }
    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
        unsafe { self.map_unchecked_mut(|this| &mut this.0) }
    }
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
    pub fn get_ref(&self) -> &T {
        &self.0
    }
}

impl<T> tokio_io::AsyncRead for Compat<T>
where
    T: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.get_pin_mut().poll_read(cx, buf)
    }

    // TODO: implement other methods for faster read
}

impl<T> tokio_io::AsyncBufRead for Compat<T>
where
    T: AsyncBufRead,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<&[u8]>> {
        self.get_pin_mut().poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.get_pin_mut().consume(amt)
    }
}

impl<T> tokio_io::AsyncWrite for Compat<T>
where
    T: AsyncWrite,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        self.get_pin_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.get_pin_mut().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        self.get_pin_mut().poll_close(cx)
    }

    // TODO: implement other methods for faster write
}

impl<T, U> tokio_executor::TypedExecutor<U> for Compat<T>
where
    T: Spawn,
    U: Future<Output = ()> + Send + 'static,
{
    fn spawn(&mut self, future: U) -> Result<(), tokio_executor::SpawnError> {
        self.get_mut().spawn(future).map_err(spawn_error_compat)
    }

    fn status(&self) -> Result<(), tokio_executor::SpawnError> {
        self.get_ref().status().map_err(spawn_error_compat)
    }
}

fn spawn_error_compat(e: SpawnError) -> tokio_executor::SpawnError {
    if e.is_shutdown() {
        tokio_executor::SpawnError::shutdown()
    } else {
        tokio_executor::SpawnError::shutdown()
    }
}
