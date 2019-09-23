use jwt::Header;
use serde::{Deserialize, Serialize};

use crate::context::AppCtx;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub token: String,
}

pub fn encode(ctx: &AppCtx, claims: &Claims) -> String {
    jwt::encode(&Header::default(), claims, ctx.secret_key.as_bytes()).unwrap()
}
