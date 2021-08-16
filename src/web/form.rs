use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;

use crate::{
    body::Body,
    error::{Error, ErrorBodyHasBeenTaken, ErrorInvalidFormContentType, Result},
    http::{
        header::{self, HeaderValue},
        Method,
    },
    web::{FromRequest, RequestParts},
};

/// An extractor that can deserialize some type from query string or body.
///
/// If the method is not `GET`, the query parameters will be parsed from the
/// body, otherwise it is like [`Query`](crate::web::Query).
///
/// If the `Content-Type` is not `application/x-www-form-urlencoded`, then a
/// `Bad Request` response will be returned.
pub struct Form<T>(pub T);

impl<T> Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Form<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait::async_trait]
impl<'a, T: DeserializeOwned> FromRequest<'a> for Form<T> {
    async fn from_request(parts: &'a RequestParts, body: &mut Option<Body>) -> Result<Self> {
        if parts.method == Method::GET {
            serde_urlencoded::from_str(parts.uri.query().unwrap_or_default())
                .map_err(Error::bad_request)
                .map(Self)
        } else {
            if parts.headers.get(header::CONTENT_TYPE)
                != Some(&HeaderValue::from_static(
                    "application/x-www-form-urlencoded",
                ))
            {
                return Err(ErrorInvalidFormContentType.into());
            }

            Ok(Self(
                serde_urlencoded::from_bytes(
                    &body
                        .take()
                        .ok_or(ErrorBodyHasBeenTaken)?
                        .into_bytes()
                        .await?,
                )
                .map_err(Error::bad_request)?,
            ))
        }
    }
}
