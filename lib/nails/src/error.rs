use failure::Fail;
use hyper::{Body, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt;

pub trait ServiceError: std::error::Error + Any + Send + Sync {
    fn status(&self) -> StatusCode;
    fn class_name(&self) -> &str;
    fn has_public_message(&self) -> bool {
        false
    }
    fn fmt_public_message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        drop(f);
        Ok(())
    }
}

#[derive(Debug)]
pub enum ErrorResponse {
    ContentTypeError(ContentTypeError),
    JsonBodyError(JsonBodyError),
    BodyError(BodyError),
    QueryError(QueryError),
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
            ContentTypeError(..) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            QueryError(..) | JsonBodyError(..) => StatusCode::BAD_REQUEST,
            BodyError(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> Option<String> {
        use ErrorResponse::*;
        match self {
            AnyError { error_code, .. } => error_code.clone(),
            _ => None,
        }
    }

    pub fn public_message(&self) -> Option<String> {
        use ErrorResponse::*;
        match self {
            AnyError { public_message, .. } => public_message.clone(),
            ContentTypeError(e) => Some(e.to_string()),
            JsonBodyError(e) => Some(e.to_string()),
            QueryError(e) => Some(e.to_string()),
            BodyError(..) => None,
        }
    }

    pub fn error(&self) -> Option<&dyn Fail> {
        use ErrorResponse::*;
        match self {
            AnyError { error, .. } => error.as_ref().map(|x| x.as_fail()),
            ContentTypeError(e) => Some(e),
            JsonBodyError(e) => Some(e),
            QueryError(e) => Some(e),
            BodyError(e) => Some(e),
        }
    }
}

impl From<ContentTypeError> for ErrorResponse {
    fn from(e: ContentTypeError) -> Self {
        ErrorResponse::ContentTypeError(e)
    }
}

impl From<JsonBodyError> for ErrorResponse {
    fn from(e: JsonBodyError) -> Self {
        ErrorResponse::JsonBodyError(e)
    }
}

impl From<QueryError> for ErrorResponse {
    fn from(e: QueryError) -> Self {
        ErrorResponse::QueryError(e)
    }
}

impl From<BodyError> for ErrorResponse {
    fn from(e: BodyError) -> Self {
        ErrorResponse::BodyError(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    message: String,
}

#[derive(Debug)]
pub struct ContentTypeError {
    pub expected: Vec<String>,
    pub got: Option<String>,
}

impl ServiceError for ContentTypeError {
    fn status(&self) -> StatusCode {
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    }
    fn class_name(&self) -> &str {
        "nails::error::ContentTypeError"
    }
    fn has_public_message(&self) -> bool {
        true
    }
    fn fmt_public_message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for ContentTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { expected, got } = self;
        write!(f, "Invalid Content-Type: expected ")?;
        if expected.is_empty() {
            write!(f, "nothing")?;
        } else if expected.len() == 1 {
            write!(f, "{:?}", expected[0])?;
        } else {
            for ct in &expected[..expected.len() - 2] {
                write!(f, "{:?}, ", ct)?;
            }
            write!(
                f,
                "{:?} and {:?}",
                expected[expected.len() - 2],
                expected[expected.len() - 1],
            )?;
        }
        if let Some(got) = got {
            write!(f, ", got {:?}", got)?;
        } else {
            write!(f, ", got nothing")?;
        }
        Ok(())
    }
}

impl std::error::Error for ContentTypeError {
    fn description(&self) -> &str {
        "Invalid Content-Type"
    }
}

#[derive(Debug)]
pub struct JsonBodyError(pub serde_json::Error);

impl ServiceError for JsonBodyError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
    fn class_name(&self) -> &str {
        "nails::error::JsonBodyError"
    }
    fn has_public_message(&self) -> bool {
        true
    }
    fn fmt_public_message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for JsonBodyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error in JSON Body: {}", self.0)
    }
}

impl std::error::Error for JsonBodyError {
    fn description(&self) -> &str {
        "Error in JSON Body"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(&self.0)
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Debug)]
pub struct BodyError(pub hyper::Error);

impl ServiceError for BodyError {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    fn class_name(&self) -> &str {
        "nails::error::BodyError"
    }
}

impl fmt::Display for BodyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error reading request body: {}", self.0)
    }
}

impl std::error::Error for BodyError {
    fn description(&self) -> &str {
        "Error reading request body"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(&self.0)
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Debug)]
pub enum QueryError {
    MultipleQuery,
    NoQuery,
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    AnyError(failure::Error),
}

impl ServiceError for QueryError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
    fn class_name(&self) -> &str {
        "nails::error::QueryError"
    }
    fn has_public_message(&self) -> bool {
        true
    }
    fn fmt_public_message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use QueryError::*;
        match self {
            MultipleQuery => write!(f, "multiple query values found"),
            NoQuery => write!(f, "no query value found"),
            ParseIntError(e) => write!(f, "{}", e),
            ParseFloatError(e) => write!(f, "{}", e),
            AnyError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for QueryError {
    fn description(&self) -> &str {
        use QueryError::*;
        match self {
            MultipleQuery => "multiple query values found",
            NoQuery => "no query value found",
            ParseIntError(e) => e.description(),
            ParseFloatError(e) => e.description(),
            AnyError(_) => "some error",
        }
    }
}

impl From<std::num::ParseIntError> for QueryError {
    fn from(e: std::num::ParseIntError) -> Self {
        QueryError::ParseIntError(e)
    }
}

impl From<std::num::ParseFloatError> for QueryError {
    fn from(e: std::num::ParseFloatError) -> Self {
        QueryError::ParseFloatError(e)
    }
}
