use nails_derive::FromRequest;

#[derive(FromRequest)]
#[nails(path = "/api/posts/{id}")]
pub enum GetPostRequest {
    Foo {}
}

fn main() {}
