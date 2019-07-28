use nails_derive::Preroute;

#[derive(Preroute)]
#[nails(path = "/api/posts/{id}")]
pub enum GetPostRequest {
    Foo {}
}

fn main() {}
