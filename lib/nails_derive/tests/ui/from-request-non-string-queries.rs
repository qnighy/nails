use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest {
    #[nails(query = 42)]
    query1: String,
}

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest2 {
    #[nails(query = b"query1rename")]
    query1: String,
}

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest3 {
    #[nails(path = 42)]
    id: String,
}

fn main() {}
