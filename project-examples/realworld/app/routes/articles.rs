use hyper::{Body, Response};
use nails::error::NailsError;
use nails::Preroute;
use serde::Serialize;

use crate::context::AppCtx;

#[derive(Debug, Preroute)]
#[nails(path = "/api/articles")]
pub(crate) struct ListArticlesRequest {
    tag: Option<String>,
    author: Option<String>,
    favorited: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListArticlesResponseBody {
    articles: Vec<Article>,
    articles_count: u64,
}

pub(crate) async fn list_articles(
    _ctx: AppCtx,
    _req: ListArticlesRequest,
) -> Result<Response<Body>, NailsError> {
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
    Ok(super::json_response(&body))
}

#[derive(Debug, Preroute)]
#[nails(path = "/api/articles/feed")]
pub(crate) struct ListFeedArticlesRequest {
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListFeedArticlesResponseBody {
    articles: Vec<Article>,
    articles_count: u64,
}

pub(crate) async fn list_feed_articles(
    _ctx: AppCtx,
    _req: ListFeedArticlesRequest,
) -> Result<Response<Body>, NailsError> {
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
    let body = ListFeedArticlesResponseBody {
        articles_count: articles.len() as u64,
        articles,
    };
    Ok(super::json_response(&body))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Article {
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
pub(crate) struct Profile {
    username: String,
    bio: String,
    image: String,
    following: bool,
}
