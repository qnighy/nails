use hyper::{Body, Method, Request, Response};
use serde::Serialize;

use nails::response::ErrorResponse;
use nails::{FromRequest, Service};

use crate::context::AppCtx;

pub fn build_route(ctx: &AppCtx) -> Service<AppCtx> {
    Service::builder()
        .add_function_route(index)
        .add_function_route(get_post)
        .add_function_route(list_tags)
        .add_function_route(list_articles)
        .finish(ctx)
}

#[derive(Debug, FromRequest)]
#[nails(path = "/")]
struct IndexRequest {
    #[nails(query)]
    a: Vec<String>,
}

async fn index(req: IndexRequest) -> Result<Response<Body>, ErrorResponse> {
    Ok(Response::new(Body::from(format!(
        "Hello, world! {:?}",
        req.a
    ))))
}

#[derive(Debug, FromRequest)]
#[nails(path = "/api/posts/{id}")]
struct GetPostRequest {
    id: u64,
}

#[derive(Debug, Serialize)]
struct GetPostBody {
    post: Post,
}

#[derive(Debug, Serialize)]
struct Post {
    body: String,
}

async fn get_post(_req: GetPostRequest) -> Result<Response<Body>, ErrorResponse> {
    let body = GetPostBody {
        post: Post {
            body: String::from("foo"),
        },
    };
    Ok(json_response(&body))
}

#[derive(Debug, FromRequest)]
#[nails(path = "/api/tags")]
struct ListTagsRequest;

#[derive(Debug, Serialize)]
struct ListTagsResponseBody {
    tags: Vec<String>,
}

async fn list_tags(_req: ListTagsRequest) -> Result<Response<Body>, ErrorResponse> {
    let body = ListTagsResponseBody {
        tags: vec![String::from("tag1"), String::from("tag2")],
    };
    Ok(json_response(&body))
}

#[derive(Debug, FromRequest)]
#[nails(path = "/api/articles")]
struct ListArticlesRequest {
    tag: Option<String>,
    author: Option<String>,
    favorited: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListArticlesResponseBody {
    articles: Vec<Article>,
    articles_count: u64,
}

async fn list_articles(_req: ListArticlesRequest) -> Result<Response<Body>, ErrorResponse> {
    let articles = vec![Article {
        slug: String::from("slug"),
        title: String::from("title"),
        description: String::from("description"),
        body: String::from("body"),
        tag_list: vec![String::from("tag2"), String::from("tag3")],
        created_at: String::from("2019-07-14T19:07:00+0900"),
        updated_at: String::from("2019-07-14T19:07:00+0900"),
        favorited: false,
        favorites_count: 0,
        author: Profile {
            username: String::from("username"),
            bio: String::from("bio"),
            image: String::from("image"),
            following: false,
        },
    }];
    let body = ListArticlesResponseBody {
        articles_count: articles.len() as u64,
        articles,
    };
    Ok(json_response(&body))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Article {
    slug: String,
    title: String,
    description: String,
    body: String,
    tag_list: Vec<String>,
    created_at: String, // TODO: DateTime
    updated_at: String, // TODO: DateTime
    favorited: bool,
    favorites_count: u64,
    author: Profile,
}

#[derive(Debug, Serialize)]
struct Profile {
    username: String,
    bio: String,
    image: String,
    following: bool,
}

fn json_response<T: Serialize>(body: &T) -> Response<Body> {
    Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}
