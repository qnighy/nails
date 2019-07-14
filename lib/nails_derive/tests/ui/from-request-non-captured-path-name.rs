use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
struct GetPostRequest {
    #[nails(path = "idd")]
    id: String,
}

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
struct GetPostRequest2 {
    #[nails(path)]
    idd: String,
}

fn main() {}
