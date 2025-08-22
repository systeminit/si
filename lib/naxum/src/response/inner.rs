use std::fmt;

use async_nats::StatusCode;

#[derive(Clone)]
pub struct Response<T> {
    head: Parts,
    body: T,
}

#[derive(Clone)]
#[non_exhaustive]
pub struct Parts {
    pub status: StatusCode,
}

impl<T> Response<T> {
    #[inline]
    pub fn new(body: T) -> Self {
        Self {
            head: Parts::new(),
            body,
        }
    }

    #[inline]
    pub fn from_parts(parts: Parts, body: T) -> Self {
        Self { head: parts, body }
    }

    #[inline]
    pub fn status(&self) -> StatusCode {
        self.head.status
    }

    #[inline]
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.head.status
    }

    #[inline]
    pub fn body(&self) -> &T {
        &self.body
    }

    #[inline]
    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    #[inline]
    pub fn into_body(self) -> T {
        self.body
    }

    #[inline]
    pub fn into_parts(self) -> (Parts, T) {
        (self.head, self.body)
    }

    #[inline]
    pub fn map<F, U>(self, f: F) -> Response<U>
    where
        F: FnOnce(T) -> U,
    {
        Response {
            head: self.head,
            body: f(self.body),
        }
    }
}

impl<T> Response<T> {
    pub fn default_internal_server_error() -> Self
    where
        T: Default,
    {
        Self {
            head: Parts {
                status: StatusCode::from_u16(500).expect("status code is in valid range"),
            },
            body: T::default(),
        }
    }

    pub fn default_ok() -> Self
    where
        T: Default,
    {
        Self {
            head: Parts {
                status: StatusCode::from_u16(200).expect("status code is in valid range"),
            },
            body: T::default(),
        }
    }

    pub fn default_bad_request() -> Self
    where
        T: Default,
    {
        Self {
            head: Parts {
                status: StatusCode::from_u16(400).expect("status code is in valid range"),
            },
            body: T::default(),
        }
    }

    pub fn default_bad_gateway() -> Self
    where
        T: Default,
    {
        Self {
            head: Parts {
                status: StatusCode::from_u16(502).expect("status code is in valid range"),
            },
            body: T::default(),
        }
    }

    pub fn default_service_unavailable() -> Self
    where
        T: Default,
    {
        Self {
            head: Parts {
                status: StatusCode::from_u16(503).expect("status code is in valid range"),
            },
            body: T::default(),
        }
    }
}

impl<T: Default> Default for Response<T> {
    #[inline]
    fn default() -> Self {
        Response::new(T::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for Response<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status())
            .finish()
    }
}

impl Parts {
    fn new() -> Self {
        Self {
            status: StatusCode::default(),
        }
    }
}

impl fmt::Debug for Parts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parts")
            .field("status", &self.status)
            .finish()
    }
}
