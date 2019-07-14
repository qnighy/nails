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

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id1}/{id2}")]
pub struct GetPostRequest3 {
    #[nails(path = "id1", path = "id2")]
    id: String,
}

fn main() {}
