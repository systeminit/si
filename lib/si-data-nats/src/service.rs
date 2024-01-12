use std::pin::Pin;

pub use async_nats::service::*;
use futures::Future;

impl ServiceExt for crate::Client {
    type Output = Pin<Box<dyn Future<Output = Result<Service, async_nats::Error>> + Send>>;

    fn add_service(&self, config: Config) -> Self::Output {
        self.inner.add_service(config)
    }

    fn service_builder(&self) -> ServiceBuilder {
        self.inner.service_builder()
    }
}
