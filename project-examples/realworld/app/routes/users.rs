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

    let jwt = tokens::encode(
        &ctx,
        &Claims {
            sub: new_user.id.to_string(),
            token: new_token,
        },
    );

    let body = CreateUserResponseBody {
        user: User {
            email: new_user.email.clone(),
            token: jwt,
            username: new_user.username.clone(),
            bio: new_user.bio.clone().unwrap_or_else(|| String::from("")),
            image: new_user.image.clone().unwrap_or_else(|| String::from("")),
        },
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

    let jwt = tokens::encode(
        &ctx,
        &Claims {
            sub: found_user.id.to_string(),
            token: found_user.token.clone(),
        },
    );

    let body = LoginResponseBody {
        user: User {
            email: found_user.email.clone(),
            token: jwt,
            username: found_user.username.clone(),
            bio: found_user.bio.clone().unwrap_or_else(|| String::from("")),
            image: found_user.image.clone().unwrap_or_else(|| String::from("")),
        },
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
