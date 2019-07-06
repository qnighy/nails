use failure::Fail;
use hyper::{Body, Response, StatusCode};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ErrorResponse {
    status: StatusCode,
    public_message: Option<String>,
    error: failure::Error,
}

impl ErrorResponse {
    // TODO: use Accept header from request
    pub fn to_response(&self) -> Response<Body> {
        Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&ErrorBody {
                    error: self.error.name().map(|x| x.to_owned()),
                    message: self
                        .public_message
                        .clone()
                        .unwrap_or_else(|| "error".to_string()),
                })
                .unwrap(),
            ))
            .unwrap()
    }
}

impl<E: Fail> From<E> for ErrorResponse {
    fn from(e: E) -> Self {
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            public_message: None,
            error: e.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    message: String,
}
