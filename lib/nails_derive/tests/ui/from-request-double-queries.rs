use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest {
    #[nails(query = "query1rename")]
    #[nails(query = "query1renamerename")]
    query1: String,
}

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest2 {
    #[nails(query = "query1rename", query = "query1renamerename")]
    query1: String,
}

fn main() {}
