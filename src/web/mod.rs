//! Commonly used as the type of extractor or response.

mod data;
mod form;
mod json;
#[cfg(feature = "multipart")]
mod multipart;
mod path;
mod query;
#[cfg(feature = "sse")]
#[cfg_attr(docsrs, doc(cfg(feature = "sse")))]
pub mod sse;
#[cfg(feature = "typed-headers")]
mod typed_header;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod websocket;

use std::convert::Infallible;

use bytes::Bytes;

/// Commonly used typed headers.
#[cfg(feature = "typed-headers")]
#[cfg_attr(docsrs, doc(cfg(feature = "typed-headers")))]
pub mod type_headers {
    pub use typed_headers::{
        Accept, AcceptEncoding, Allow, AuthScheme, Authorization, ContentCoding, ContentEncoding,
        ContentLength, ContentType, Credentials, Host, HttpDate, ProxyAuthorization, Quality,
        QualityItem, RetryAfter, Token68,
    };
}

pub use data::Data;
pub use form::Form;
pub use json::Json;
#[cfg(feature = "multipart")]
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
pub use multipart::{Field, Multipart};
pub use path::Path;
pub use query::Query;
#[cfg(feature = "typed-headers")]
#[cfg_attr(docsrs, doc(cfg(feature = "typed-headers")))]
pub use typed_header::TypedHeader;

use crate::{
    body::Body,
    error::{Error, ErrorBodyHasBeenTaken, Result},
    http::{header::HeaderMap, Extensions, Method, StatusCode, Uri, Version},
    response::Response,
};

/// Component parts of an HTTP Request
pub struct RequestParts {
    /// The request’s method
    pub method: Method,

    /// The request’s URI
    pub uri: Uri,

    /// The request’s version
    pub version: Version,

    /// The request’s headers
    pub headers: HeaderMap,

    /// The request’s extensions
    pub extensions: Extensions,
}

/// Types that can be created from requests.
#[async_trait::async_trait]
pub trait FromRequest<'a>: Sized {
    /// Perform the extraction.
    async fn from_request(parts: &'a RequestParts, body: &mut Option<Body>) -> Result<Self>;
}

/// Trait for generating responses.
///
/// Types that implement [IntoResponse] can be returned from endpoints/handlers.
pub trait IntoResponse {
    /// Consume itself and return [`Response`].
    fn into_response(self) -> Result<Response>;
}

impl IntoResponse for Response {
    fn into_response(self) -> Result<Response> {
        Ok(self)
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(self.into())
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(Body::empty())
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> Result<Response> {
        Response::builder().body(Body::empty())
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Result<Response> {
        Response::builder().status(self).body(Body::empty())
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, T) {
    fn into_response(self) -> Result<Response> {
        let mut resp = self.1.into_response()?;
        resp.set_status(self.0);
        Ok(resp)
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, HeaderMap, T) {
    fn into_response(self) -> Result<Response> {
        let mut resp = self.2.into_response()?;
        resp.set_status(self.0);
        resp.headers_mut().extend(self.1.into_iter());
        Ok(resp)
    }
}

impl<T: IntoResponse, E: Into<Error>> IntoResponse for Result<T, E> {
    fn into_response(self) -> Result<Response> {
        self.map_err(Into::into)
            .and_then(IntoResponse::into_response)
    }
}

/// An HTML response.
pub struct Html<T>(pub T);

impl<T: Into<String>> IntoResponse for Html<T> {
    fn into_response(self) -> Result<Response> {
        Response::builder()
            .content_type("text/html")
            .body(self.0.into().into())
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for &'a RequestParts {
    async fn from_request(parts: &'a RequestParts, _body: &mut Option<Body>) -> Result<Self> {
        Ok(parts)
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for &'a Uri {
    async fn from_request(parts: &'a RequestParts, _body: &mut Option<Body>) -> Result<Self> {
        Ok(&parts.uri)
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Method {
    async fn from_request(parts: &'a RequestParts, _body: &mut Option<Body>) -> Result<Self> {
        Ok(parts.method.clone())
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Version {
    async fn from_request(parts: &'a RequestParts, _body: &mut Option<Body>) -> Result<Self> {
        Ok(parts.version)
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for &'a HeaderMap {
    async fn from_request(parts: &'a RequestParts, _body: &mut Option<Body>) -> Result<Self> {
        Ok(&parts.headers)
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Body {
    async fn from_request(_parts: &'a RequestParts, body: &mut Option<Body>) -> Result<Self> {
        Ok(body.take().ok_or(ErrorBodyHasBeenTaken)?)
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for String {
    async fn from_request(_parts: &'a RequestParts, body: &mut Option<Body>) -> Result<Self> {
        String::from_utf8(
            body.take()
                .ok_or(ErrorBodyHasBeenTaken)?
                .into_bytes()
                .await?
                .to_vec(),
        )
        .map_err(Error::bad_request)
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Bytes {
    async fn from_request(_parts: &'a RequestParts, body: &mut Option<Body>) -> Result<Self> {
        Ok(body
            .take()
            .ok_or(ErrorBodyHasBeenTaken)?
            .into_bytes()
            .await?)
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Vec<u8> {
    async fn from_request(_parts: &'a RequestParts, body: &mut Option<Body>) -> Result<Self> {
        Ok(body
            .take()
            .ok_or(ErrorBodyHasBeenTaken)?
            .into_bytes()
            .await?
            .to_vec())
    }
}

#[async_trait::async_trait]
impl<'a, T: FromRequest<'a>> FromRequest<'a> for Option<T> {
    async fn from_request(parts: &'a RequestParts, body: &mut Option<Body>) -> Result<Self> {
        Ok(T::from_request(parts, body).await.ok())
    }
}
