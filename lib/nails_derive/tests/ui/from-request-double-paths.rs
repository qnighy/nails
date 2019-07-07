use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
#[nails(path = "/api/posts/{idd}")]
pub struct GetPostRequest {}

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}", path = "/api/posts/{idd}")]
pub struct GetPostRequest2 {}

fn main() {}
