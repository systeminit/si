use std::{convert::Infallible, ops};

use async_trait::async_trait;

use crate::message::Head;

use super::{FromMessageHead, FromRef};

#[derive(Debug, Default, Clone, Copy)]
pub struct State<S>(pub S);

#[async_trait]
impl<OuterState, InnerState> FromMessageHead<OuterState> for State<InnerState>
where
    InnerState: FromRef<OuterState>,
    OuterState: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_message_head(
        _head: &mut Head,
        state: &OuterState,
    ) -> Result<Self, Self::Rejection> {
        let inner_state = InnerState::from_ref(state);
        Ok(Self(inner_state))
    }
}

impl<S> ops::Deref for State<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> ops::DerefMut for State<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
