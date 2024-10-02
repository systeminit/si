use crate::{Message, MessageHead};

pub trait ForSubject<R>
where
    R: MessageHead,
{
    fn call(&mut self, req: &mut Message<R>);
}

#[derive(Clone, Debug, Default)]
pub struct DefaultForSubject {}

impl DefaultForSubject {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<R> ForSubject<R> for DefaultForSubject
where
    R: MessageHead,
{
    fn call(&mut self, _req: &mut Message<R>) {}
}
