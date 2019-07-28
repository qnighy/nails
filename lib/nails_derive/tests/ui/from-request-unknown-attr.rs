use nails_derive::Preroute;

#[derive(Preroute)]
#[nails(path = "/api/posts/{id}", foo)]
pub struct GetPostRequest {}

#[derive(Preroute)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest2 {
    #[nails(query, foo)]
    query1: String,
}

fn main() {}
