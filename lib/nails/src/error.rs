use failure::Fail;
use hyper::{Body, Response, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ErrorResponse {
    AnyError {
        status: StatusCode,
        error_code: Option<String>,
        public_message: Option<String>,
        error: Option<failure::Error>,
    },
}

impl ErrorResponse {
    // TODO: use Accept header from request
    pub fn to_response(&self) -> Response<Body> {
        Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&ErrorBody {
                    error: self.error().and_then(|e| e.name().map(|x| x.to_owned())),
                    message: self
                        .public_message()
                        .clone()
                        .unwrap_or_else(|| "error".to_string()),
                })
                .unwrap(),
            ))
            .unwrap()
    }

    pub fn status(&self) -> StatusCode {
        use ErrorResponse::*;
        match self {
            AnyError { status, .. } => *status,
        }
    }

    pub fn error_code(&self) -> Option<String> {
        use ErrorResponse::*;
        match self {
            AnyError { error_code, .. } => error_code.clone(),
        }
    }

    pub fn public_message(&self) -> Option<String> {
        use ErrorResponse::*;
        match self {
            AnyError { public_message, .. } => public_message.clone(),
        }
    }

    pub fn error(&self) -> Option<&dyn Fail> {
        use ErrorResponse::*;
        match self {
            AnyError { error, .. } => error.as_ref().map(|x| x.as_fail()),
        }
    }
}

impl<E: Fail> From<E> for ErrorResponse {
    fn from(e: E) -> Self {
        ErrorResponse::AnyError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error_code: None,
            public_message: None,
            error: Some(e.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    message: String,
}
