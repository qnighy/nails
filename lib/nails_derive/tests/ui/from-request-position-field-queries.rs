use nails_derive::Preroute;

#[derive(Preroute)]
#[nails(path = "/api/posts/{id}")]
pub struct GetPostRequest(
    #[nails(query)]
    String,
);

fn main() {}
