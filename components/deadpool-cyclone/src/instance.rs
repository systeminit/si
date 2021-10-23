use std::result;

use async_trait::async_trait;

pub use local_uds::{LocalUdsInstance, LocalUdsInstanceSpec};

mod local_uds;

#[async_trait]
pub trait InstanceSpec {
    type Instance: Instance<Error = Self::Error>;
    type Error;

    async fn spawn(&self) -> result::Result<Self::Instance, Self::Error>;
}

pub trait InstanceSpecBuilder: Default {
    type Spec;
    type Error;

    fn build(&self) -> result::Result<Self::Spec, Self::Error>;
}

#[async_trait]
pub trait Instance {
    type SpecBuilder: Default;
    type Error;

    fn spec() -> Self::SpecBuilder {
        Self::SpecBuilder::default()
    }

    async fn ensure_healthy(&mut self) -> result::Result<(), Self::Error>;

    async fn terminate(mut self) -> result::Result<(), Self::Error>;
}

// async fn spawn<B, E, I, S>(builder: &B) -> Result<impl Instance<Error = E>, E>
// where
//     B: InstanceSpecBuilder<Spec = S, Error = E>,
//     S: InstanceSpec<Error = E, Instance = I>,
//     I: Instance<SpecBuilder = B, Error = E>,
// {
//     builder.build()?.spawn().await
// }
