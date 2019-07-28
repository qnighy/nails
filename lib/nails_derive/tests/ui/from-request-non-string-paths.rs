use nails_derive::Preroute;

#[derive(Preroute)]
#[nails(path = 42)]
pub struct GetPostRequest {}

#[derive(Preroute)]
#[nails(path = b"/api/posts/{id}")]
pub struct GetPostRequest2 {}

#[derive(Preroute)]
#[nails(path)]
pub struct GetPostRequest3 {}

fn main() {}
