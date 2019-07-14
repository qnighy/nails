use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest {
    #[nails(path)]
    id: String,
    #[nails(path = "id")]
    id2: String,
}

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest2 {
    #[nails(path = "id")]
    id2: String,
    id: String,
}

fn main() {}
