use axum::{response::IntoResponse, Json};
use std::fmt;
use std::sync::Arc;

use http::StatusCode;
use svc_error::Error as SvcError;

pub struct ErrorKindProperties {
    status: StatusCode,
    kind: &'static str,
    title: &'static str,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ErrorKind {
    MissingMaxmind,
    RefererError,
    ObjectReadingError,
    MissingAudienceSetting,
    SigningError,
    BackendNotFound,
    SigningForbidden,
}

impl ErrorKind {
    pub fn status(self) -> StatusCode {
        let properties: ErrorKindProperties = self.into();
        properties.status
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let properties: ErrorKindProperties = self.to_owned().into();
        write!(f, "{}", properties.title)
    }
}

impl From<ErrorKind> for ErrorKindProperties {
    fn from(val: ErrorKind) -> Self {
        match val {
            ErrorKind::MissingMaxmind => ErrorKindProperties {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                kind: "missing_maxmind",
                title: "missing maxminddb reader",
            },
            ErrorKind::RefererError => ErrorKindProperties {
                status: StatusCode::BAD_REQUEST,
                kind: "error_reading_referer",
                title: "Error reading 'REFERER' header",
            },
            ErrorKind::ObjectReadingError => ErrorKindProperties {
                status: StatusCode::FORBIDDEN,
                kind: "error_reading_object",
                title: "Error reading object in the set",
            },
            ErrorKind::MissingAudienceSetting => ErrorKindProperties {
                status: StatusCode::NOT_FOUND,
                kind: "missing_audience_setting",
                title:
                    "Error reading an object using Set API: Audience settings for bucket not found",
            },
            ErrorKind::SigningError => ErrorKindProperties {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                kind: "signing_error",
                title: "Error signing a request",
            },
            ErrorKind::BackendNotFound => ErrorKindProperties {
                status: StatusCode::NOT_FOUND,
                kind: "backend_not_found",
                title: "Error signing a request: Backend not found",
            },
            ErrorKind::SigningForbidden => ErrorKindProperties {
                status: StatusCode::FORBIDDEN,
                kind: "signing_forbidden",
                title: "Access denied",
            },
        }
    }
}

pub struct Error {
    kind: ErrorKind,
    err: Option<Arc<anyhow::Error>>,
}

impl Error {
    pub fn new(kind: ErrorKind, err: Option<anyhow::Error>) -> Self {
        Self {
            kind,
            err: err.map(Arc::new),
        }
    }

    pub fn status(&self) -> StatusCode {
        self.kind.status()
    }

    pub fn error_kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn detail(&self) -> String {
        match &self.err {
            Some(s) => s.to_string(),
            None => String::new(),
        }
    }

    pub fn to_svc_error(&self) -> SvcError {
        let properties: ErrorKindProperties = self.kind.into();

        SvcError::builder()
            .status(properties.status)
            .kind(properties.kind, properties.title)
            .detail(&self.detail())
            .build()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("source", &self.err)
            .finish()
    }
}

pub trait ErrorExt<T> {
    fn error(self, kind: ErrorKind) -> Result<T, Error>;
}

impl<T, E: Into<anyhow::Error>> ErrorExt<T> for Result<T, E> {
    fn error(self, kind: ErrorKind) -> Result<T, Error> {
        self.map_err(|source| Error::new(kind, Some(source.into())))
    }
}

pub trait ErrorKindExt {
    fn kind(self, kind: ErrorKind) -> Error;
}

impl ErrorKindExt for anyhow::Error {
    fn kind(self, kind: ErrorKind) -> Error {
        Error::new(kind, Some(self))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let err = self.to_svc_error();

        let mut r = (self.status(), Json(err)).into_response();
        r.extensions_mut().insert(self.error_kind());

        r
    }
}
