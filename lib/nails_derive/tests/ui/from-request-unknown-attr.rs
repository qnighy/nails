use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}", foo)]
pub struct GetPostRequest {}

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest2 {
    #[nails(query, foo)]
    query1: String,
}

fn main() {}
