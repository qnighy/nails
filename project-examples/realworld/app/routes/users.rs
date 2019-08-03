use hyper::{Body, Response};
use nails::response::ErrorResponse;
use nails::Preroute;
use serde::Serialize;

use crate::context::AppCtx;

#[derive(Debug, Preroute)]
#[nails(path = "/api/users", method = "POST")]
pub(crate) struct CreateUserRequest;

#[derive(Debug, Serialize)]
pub(crate) struct CreateUserResponseBody {}

pub(crate) async fn create_user(
    _ctx: AppCtx,
    _req: CreateUserRequest,
) -> Result<Response<Body>, ErrorResponse> {
    let body = CreateUserResponseBody {};
    Ok(super::json_response(&body))
}
