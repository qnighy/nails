use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = 42)]
pub struct GetPostRequest {}

#[derive(FromRequest)]
#[nails(path = b"/api/posts/{id}")]
pub struct GetPostRequest2 {}

#[derive(FromRequest)]
#[nails(path)]
pub struct GetPostRequest3 {}

fn main() {}
