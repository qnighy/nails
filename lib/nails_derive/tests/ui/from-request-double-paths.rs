use nails_derive::Preroute;

#[derive(Preroute)]
#[nails(path = "/api/posts/{id}")]
#[nails(path = "/api/posts/{idd}")]
pub struct GetPostRequest {}

#[derive(Preroute)]
#[nails(path = "/api/posts/{id}", path = "/api/posts/{idd}")]
pub struct GetPostRequest2 {}

fn main() {}
