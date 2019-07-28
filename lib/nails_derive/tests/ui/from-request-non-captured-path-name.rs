use nails_derive::Preroute;

#[derive(Preroute)]
#[nails(path = "/api/posts/{id}")]
struct GetPostRequest {
    #[nails(path = "idd")]
    id: String,
}

#[derive(Preroute)]
#[nails(path = "/api/posts/{id}")]
struct GetPostRequest2 {
    #[nails(path)]
    idd: String,
}

fn main() {}
