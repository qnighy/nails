use hyper::{Body, Response};
use nails::request::JsonBody;
use nails::response::ErrorResponse;
use nails::Preroute;
use serde::{Deserialize, Serialize};

use crate::context::AppCtx;

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
pub(crate) struct CreateUserResponseBody {}

pub(crate) async fn create_user(
    _ctx: AppCtx,
    req: CreateUserRequest,
) -> Result<Response<Body>, ErrorResponse> {
    eprintln!("body = {:?}", req.body);
    let body = CreateUserResponseBody {};
    Ok(super::json_response(&body))
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct NewUser {
    username: String,
    email: String,
    password: String,
}
