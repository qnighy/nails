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

pub trait ServiceErrorExt: ServiceError {
    fn public_message(&self) -> Option<PublicMessage<'_, Self>> {
        if self.has_public_message() {
            Some(PublicMessage(self))
        } else {
            None
        }
    }
}
impl<T: ServiceError + ?Sized> ServiceErrorExt for T {}

pub struct PublicMessage<'a, E: ServiceError + ?Sized>(&'a E);

impl<E: ServiceError + ?Sized> fmt::Display for PublicMessage<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_public_message(f)
    }
}

#[derive(Debug)]
pub enum NailsError {
    ContentTypeError(ContentTypeError),
    JsonBodyError(JsonBodyError),
    BodyError(BodyError),
    QueryError(QueryError),
    AnyError(Box<dyn ServiceError>),
}

impl NailsError {
    // TODO: use Accept header from request
    pub fn to_response(&self) -> Response<Body> {
        Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_string(&ErrorBody {
                    error: self.class_name().to_owned(),
                    message: self
                        .public_message()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "error".to_string()),
                })
                .unwrap(),
            ))
            .unwrap()
    }
}

impl ServiceError for NailsError {
    fn status(&self) -> StatusCode {
        use NailsError::*;
        match self {
            ContentTypeError(e) => e.status(),
            JsonBodyError(e) => e.status(),
            BodyError(e) => e.status(),
            QueryError(e) => e.status(),
            AnyError(e) => e.status(),
        }
    }
    fn class_name(&self) -> &str {
        use NailsError::*;
        match self {
            ContentTypeError(e) => e.class_name(),
            JsonBodyError(e) => e.class_name(),
            BodyError(e) => e.class_name(),
            QueryError(e) => e.class_name(),
            AnyError(e) => e.class_name(),
        }
    }
    fn has_public_message(&self) -> bool {
        use NailsError::*;
        match self {
            ContentTypeError(e) => e.has_public_message(),
            JsonBodyError(e) => e.has_public_message(),
            BodyError(e) => e.has_public_message(),
            QueryError(e) => e.has_public_message(),
            AnyError(e) => e.has_public_message(),
        }
    }
    fn fmt_public_message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use NailsError::*;
        match self {
            ContentTypeError(e) => e.fmt_public_message(f),
            JsonBodyError(e) => e.fmt_public_message(f),
            BodyError(e) => e.fmt_public_message(f),
            QueryError(e) => e.fmt_public_message(f),
            AnyError(e) => e.fmt_public_message(f),
        }
    }
}

impl std::error::Error for NailsError {
    fn description(&self) -> &str {
        use NailsError::*;
        match self {
            ContentTypeError(e) => e.description(),
            JsonBodyError(e) => e.description(),
            BodyError(e) => e.description(),
            QueryError(e) => e.description(),
            AnyError(e) => e.description(),
        }
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use NailsError::*;
        match self {
            ContentTypeError(e) => e.source(),
            JsonBodyError(e) => e.source(),
            BodyError(e) => e.source(),
            QueryError(e) => e.source(),
            AnyError(e) => e.source(),
        }
    }
}

impl fmt::Display for NailsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use NailsError::*;
        match self {
            ContentTypeError(e) => e.fmt(f),
            JsonBodyError(e) => e.fmt(f),
            BodyError(e) => e.fmt(f),
            QueryError(e) => e.fmt(f),
            AnyError(e) => e.fmt(f),
        }
    }
}

impl From<ContentTypeError> for NailsError {
    fn from(e: ContentTypeError) -> Self {
        NailsError::ContentTypeError(e)
    }
}

impl From<JsonBodyError> for NailsError {
    fn from(e: JsonBodyError) -> Self {
        NailsError::JsonBodyError(e)
    }
}

impl From<QueryError> for NailsError {
    fn from(e: QueryError) -> Self {
        NailsError::QueryError(e)
    }
}

impl From<BodyError> for NailsError {
    fn from(e: BodyError) -> Self {
        NailsError::BodyError(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorBody {
    error: String,
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
