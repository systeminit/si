use std::{
    convert::Infallible,
    str,
};

use async_nats::{
    HeaderMap,
    Subject,
};
use async_trait::async_trait;
use bytes::Bytes;

use super::{
    FromMessage,
    FromMessageHead,
    rejection::{
        NoReplyRejection,
        StringRejection,
    },
};
use crate::{
    extract::rejection::InvalidUtf8,
    message::{
        Extensions,
        Head,
        Message,
        MessageHead,
    },
};

#[async_trait]
impl<S, R> FromMessage<S, R> for Message<R>
where
    S: Send + Sync,
    R: MessageHead + Send,
{
    type Rejection = Infallible;

    async fn from_message(req: Message<R>, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(req)
    }
}

#[async_trait]
impl<S> FromMessageHead<S> for Subject {
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(head.subject.clone())
    }
}

pub struct Reply(pub Option<Subject>);

#[async_trait]
impl<S> FromMessageHead<S> for Reply {
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(head.reply.clone()))
    }
}

pub struct RequiredReply(pub Subject);

#[async_trait]
impl<S> FromMessageHead<S> for RequiredReply {
    type Rejection = NoReplyRejection;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(head.reply.clone().ok_or(NoReplyRejection)?))
    }
}

pub struct Headers(pub Option<HeaderMap>);

#[async_trait]
impl<S> FromMessageHead<S> for Headers {
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(head.headers.clone()))
    }
}

pub struct StatusCode(pub Option<async_nats::StatusCode>);

#[async_trait]
impl<S> FromMessageHead<S> for StatusCode {
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(head.status))
    }
}

pub struct Length(pub usize);

#[async_trait]
impl<S> FromMessageHead<S> for Length {
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(head.length))
    }
}

#[async_trait]
impl<S, R> FromMessage<S, R> for Bytes
where
    S: Send + Sync,
    R: MessageHead + Send + 'static,
{
    type Rejection = Infallible;

    async fn from_message(req: Message<R>, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(req.into_parts().1)
    }
}

#[async_trait]
impl<S, R> FromMessage<S, R> for String
where
    S: Send + Sync,
    R: MessageHead + Send + 'static,
{
    type Rejection = StringRejection;

    async fn from_message(req: Message<R>, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_message(req, state).await.unwrap();
        let string = str::from_utf8(&bytes)
            .map_err(InvalidUtf8::from_err)?
            .to_owned();

        Ok(string)
    }
}

#[async_trait]
impl<S> FromMessageHead<S> for Head
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(head.clone())
    }
}

#[async_trait]
impl<S> FromMessageHead<S> for Extensions
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(head.extensions.clone())
    }
}
