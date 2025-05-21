use tower::Service;

use crate::{
    error_handling::HandleError,
    make_service::IntoMakeService,
    message::Message,
};

pub trait GenericServiceExt<R>: Service<R> + Sized {
    fn into_make_service(self) -> IntoMakeService<Self>;

    fn handle_error<F, T>(self, f: F) -> HandleError<Self, F, T> {
        HandleError::new(self, f)
    }
}

impl<S, R> GenericServiceExt<R> for S
where
    S: Service<R> + Sized,
{
    fn into_make_service(self) -> IntoMakeService<Self> {
        IntoMakeService::new(self)
    }
}
