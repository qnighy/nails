use diesel::prelude::*;
use rand::prelude::*;

use hyper::{Body, Response};
use nails::error::NailsError;
use nails::request::JsonBody;
use nails::Preroute;
use serde::{Deserialize, Serialize};

use crate::context::AppCtx;
use crate::models;
use crate::tokens::{self, Claims};

#[derive(Debug, Preroute)]
#[nails(path = "/api/users", method = "POST")]
pub(crate) struct CreateUserRequest {
    #[nails(body)]
    body: JsonBody<CreateUserRequestBody>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateUserRequestBody {
    user: NewUser,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateUserResponseBody {
    user: User,
}

pub(crate) async fn create_user(
    ctx: AppCtx,
    req: CreateUserRequest,
) -> Result<Response<Body>, NailsError> {
    use crate::schema::users::dsl::*;

    let mut rng = rand::thread_rng();
    let new_token = {
        let mut buf = [0; 32];
        rng.fill_bytes(&mut buf);
        base64::encode(&buf)
    };
    let user = &req.body.0.user;

    // TODO: async
    let conn = ctx.db.get().unwrap(); // TODO: handle errors

    let new_user = models::NewUser {
        email: &user.email,
        token: &new_token,
        username: &user.username,
        bio: None,
        image: None,
    };
    let new_user = new_user
        .insert_into(users)
        .get_result::<models::User>(&conn)
        .unwrap(); // TODO: handle errors

    let body = CreateUserResponseBody {
        user: User::from_model(&ctx, new_user),
    };
    Ok(super::json_response(&body))
}

#[derive(Debug, Preroute)]
#[nails(path = "/api/users/login", method = "POST")]
pub(crate) struct LoginRequest {
    #[nails(body)]
    body: JsonBody<LoginRequestBody>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginRequestBody {
    user: LoginUser,
}

#[derive(Debug, Serialize)]
pub(crate) struct LoginResponseBody {
    user: User,
}

pub(crate) async fn login(
    ctx: AppCtx,
    req: LoginRequest,
) -> Result<Response<Body>, NailsError> {
    use crate::schema::users::dsl::*;

    let login_user = &req.body.0.user;

    // TODO: async
    let conn = ctx.db.get().unwrap(); // TODO: handle errors

    // TODO: handle errors
    let found_user = users.filter(email.eq(&login_user.email)).first::<models::User>(&conn).unwrap();

    // TODO: check password

    let body = LoginResponseBody {
        user: User::from_model(&ctx, found_user),
    };
    Ok(super::json_response(&body))
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct NewUser {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LoginUser {
    email: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct User {
    email: String,
    token: String,
    username: String,
    bio: String,
    image: String,
}

impl User {
    fn from_model(ctx: &AppCtx, user: models::User) -> Self {
        let jwt = tokens::encode(
            ctx,
            &Claims {
                sub: user.id.to_string(),
                token: user.token,
            },
        );
        Self {
            email: user.email,
            token: jwt,
            username: user.username,
            bio: user.bio.unwrap_or_else(String::default),
            image: user.image.unwrap_or_else(String::default),
        }
    }
}
