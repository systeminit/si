/// Private API.
#[doc(hidden)]
#[macro_export]
macro_rules! __log_rejection {
    (
        rejection_type = $ty:ident,
        body_text = $body_text:expr,
        status = $status:expr,
    ) => {
        tracing::event!(
            target: "naxum::rejection",
            tracing::Level::TRACE,
            status = $status.as_u16(),
            body = $body_text,
            rejection_type = std::any::type_name::<$ty>(),
            "rejecting message",
        );
    };
}

#[macro_export]
macro_rules! define_rejection {
    (
        #[status_code = $status_code:expr]
        #[body = $body:expr]
        $(#[$m:meta])*
        pub struct $name:ident;
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        #[non_exhaustive]
        pub struct $name;

        impl $crate::response::IntoResponse for $name {
            fn into_response(self) -> $crate::response::Response {
                let status = self.status();
                $crate::__log_rejection!(
                    rejection_type = $name,
                    body_text = $body,
                    status = status,
                );
                (status, $body).into_response()
            }
        }

        impl $name {
            /// Get the response body text used for this rejection.
            pub fn body_text(&self) -> String {
                $body.into()
            }

            /// Get the status code used for this rejection.
            pub fn status(&self) -> $crate::StatusCode {
                $crate::StatusCode::from_u16($status_code).expect("status code is valid")
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", $body)
            }
        }

        impl std::error::Error for $name {}

        impl Default for $name {
            fn default() -> Self {
                Self
            }
        }
    };

    (
        #[status_code = $status_code:expr]
        #[body = $body:expr]
        $(#[$m:meta])*
        pub struct $name:ident(Error);
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        pub struct $name(pub(crate) $crate::Error);

        impl $name {
            pub(crate) fn from_err<E>(err: E) -> Self
            where
                E: Into<$crate::BoxError>,
            {
                Self($crate::Error::new(err))
            }
        }

        impl $crate::response::IntoResponse for $name {
            fn into_response(self) -> $crate::response::Response {
                let status = self.status();
                $crate::__log_rejection!(
                    rejection_type = $name,
                    body_text = $body,
                    status = status,
                );
                (status, $body).into_response()
            }
        }

        impl $name {
            /// Get the response body text used for this rejection.
            pub fn body_text(&self) -> String {
                $body.into()
            }

            /// Get the status code used for this rejection.
            pub fn status(&self) -> $crate::StatusCode {
                $crate::StatusCode::from_u16($status_code).expect("status code is valid")
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", $body)
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.0)
            }
        }
    };
}

#[macro_export]
macro_rules! composite_rejection {
    (
        $(#[$m:meta])*
        pub enum $name:ident {
            $($variant:ident),+
            $(,)?
        }
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        #[non_exhaustive]
        pub enum $name {
            $(
                #[allow(missing_docs)]
                $variant($variant)
            ),+
        }

        impl $crate::response::IntoResponse for $name {
            fn into_response(self) -> $crate::response::Response {
                match self {
                    $(
                        Self::$variant(inner) => inner.into_response(),
                    )+
                }
            }
        }

        impl $name {
            /// Get the response body text used for this rejection.
            pub fn body_text(&self) -> String {
                match self {
                    $(
                        Self::$variant(inner) => inner.body_text(),
                    )+
                }
            }

            /// Get the status code used for this rejection.
            pub fn status(&self) -> $crate::StatusCode {
                match self {
                    $(
                        Self::$variant(inner) => inner.status(),
                    )+
                }
            }
        }

        $(
            impl From<$variant> for $name {
                fn from(inner: $variant) -> Self {
                    Self::$variant(inner)
                }
            }
        )+

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant(inner) => write!(f, "{inner}"),
                    )+
                }
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    $(
                        Self::$variant(inner) => inner.source(),
                    )+
                }
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

macro_rules! all_the_tuples_no_last_special_case {
    ($name:ident) => {
        $name!(T1);
        $name!(T1, T2);
        $name!(T1, T2, T3);
        $name!(T1, T2, T3, T4);
        $name!(T1, T2, T3, T4, T5);
        $name!(T1, T2, T3, T4, T5, T6);
        $name!(T1, T2, T3, T4, T5, T6, T7);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
    };
}
