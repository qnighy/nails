use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest(
    #[nails(query)]
    String,
);

fn main() {}
