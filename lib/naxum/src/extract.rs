use std::convert::Infallible;

use async_trait::async_trait;

use crate::{message::Head, response::IntoResponse, MessageHead};

mod message_parts;
pub mod rejection;
mod state;
mod tuple;

pub use self::state::State;

mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaHead {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaMessage {}
}

#[async_trait]
pub trait FromMessageHead<S>: Sized {
    type Rejection: IntoResponse;

    async fn from_message_head(head: &mut Head, state: &S) -> Result<Self, Self::Rejection>;
}

#[async_trait]
pub trait FromMessage<S, R, M = private::ViaMessage>: Sized
where
    R: MessageHead,
{
    type Rejection: IntoResponse;

    async fn from_message(req: R, state: &S) -> Result<Self, Self::Rejection>;
}

#[async_trait]
impl<S, R, T> FromMessage<S, R, private::ViaHead> for T
where
    S: Send + Sync,
    R: MessageHead + Send + 'static,
    T: FromMessageHead<S>,
{
    type Rejection = <Self as FromMessageHead<S>>::Rejection;

    async fn from_message(req: R, state: &S) -> Result<Self, Self::Rejection> {
        let (mut head, _payload) = req.into_parts();
        Self::from_message_head(&mut head, state).await
    }
}

#[async_trait]
impl<S, T> FromMessageHead<S> for Option<T>
where
    S: Send + Sync,
    T: FromMessageHead<S>,
{
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, state: &S) -> Result<Self, Self::Rejection> {
        Ok(T::from_message_head(head, state).await.ok())
    }
}

#[async_trait]
impl<S, R, T> FromMessage<S, R> for Option<T>
where
    S: Send + Sync,
    R: MessageHead + Send + 'static,
    T: FromMessage<S, R>,
{
    type Rejection = Infallible;

    async fn from_message(req: R, state: &S) -> Result<Self, Self::Rejection> {
        Ok(T::from_message(req, state).await.ok())
    }
}

#[async_trait]
impl<S, T> FromMessageHead<S> for Result<T, T::Rejection>
where
    T: FromMessageHead<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_message_head(head: &mut Head, state: &S) -> Result<Self, Self::Rejection> {
        Ok(T::from_message_head(head, state).await)
    }
}

#[async_trait]
impl<S, R, T> FromMessage<S, R> for Result<T, T::Rejection>
where
    S: Send + Sync,
    R: MessageHead + Send + 'static,
    T: FromMessageHead<S>,
{
    type Rejection = Infallible;

    async fn from_message(req: R, state: &S) -> Result<Self, Self::Rejection> {
        Ok(T::from_message(req, state).await)
    }
}

pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl<T> FromRef<T> for T
where
    T: Clone,
{
    fn from_ref(input: &T) -> Self {
        input.clone()
    }
}
