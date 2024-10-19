#![allow(deprecated)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate url;

extern crate reqwest;

pub mod apis;
pub mod client;
pub mod models;

pub use client::ApiClient;


pub type Result<T> = std::result::Result<T, Error>;
pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub struct Error {
    #[source]
    kind: ErrorKind,
}
impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(source: E) -> Self {
        Self { kind: source.into() }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[non_exhaustive]
pub enum ErrorKind {
    Reqwest(#[from] reqwest::Error),
    Serde(#[from] serde_json::Error),
    Io(#[from] std::io::Error),
    Url(#[from] ::url::ParseError),
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
    Response(#[from] ErrorResponse),
    Other(BoxError),
}

#[derive(Debug, thiserror::Error)]
pub struct ErrorResponse {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub source: Option<BoxError>,
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, &self.content)?;
        if let Some(ref err) = self.source {
            write!(f, " -- {err}")?;
        }
        Ok(())
    }
}

pub(crate) fn urlencode(s: impl AsRef<str>) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}
