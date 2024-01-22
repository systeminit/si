use std::result;

use async_trait::async_trait;

pub mod cyclone;

/// A specification sufficient to spawn [`Instance`]s.
///
/// Specs hold any behavioral decisions, configuration details, etc. and can be thought of as a
/// factory to build multiple Instances.
#[async_trait]
pub trait Spec {
    /// The type which implements the [`Instance`] trait and will be generated for each
    /// [`Self::spawn`].
    type Instance: Instance<Error = Self::Error>;
    /// Error type returned for errors when calling member methods.
    type Error;

    /// Performs setup activities to prepare the host to create [Instance]s.
    async fn setup(&mut self) -> result::Result<(), Self::Error>;
    /// Creates and launches an [`Instance`].
    ///
    /// NOTE: the method is non-consuming so that multiple Instances can be spawned from the same
    /// `Spec`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::error::Error;
    /// use deadpool_cyclone::instance::{Instance, Spec, SpecBuilder};
    /// # #[derive(Default)]
    /// # struct Builder { coolness: usize };
    /// # impl SpecBuilder for Builder {
    /// #       type Spec = MySpec;
    /// #       type Error = Box<dyn Error + 'static>;
    /// #       fn build(&self) -> Result<Self::Spec, Self::Error> {
    /// #           Ok(Self::Spec { coolness: self.coolness })
    /// #       }
    /// # }
    /// # impl Builder {
    /// #     fn coolness(&mut self, c: usize) -> &mut Self { self.coolness = c; self }
    /// # }
    /// # #[async_trait::async_trait]
    /// # impl Instance for MyInstance {
    /// #     type SpecBuilder = Builder;
    /// #     type Error = SpawnError;
    /// #     async fn ensure_healthy(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn terminate(mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    ///
    /// #[derive(Debug, thiserror::Error, PartialEq)]
    /// #[error("failed to spawn")]
    /// struct SpawnError;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct MyInstance { coolness: usize }
    ///
    /// struct MySpec { coolness: usize }
    ///
    /// #[async_trait::async_trait]
    /// impl Spec for MySpec {
    ///     type Instance = MyInstance;
    ///     type Error = SpawnError;
    ///
    ///     async fn spawn(&self) -> Result<Self::Instance, Self::Error> {
    ///         Ok(Self::Instance { coolness: self.coolness })
    ///     }
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// // A successful termination returns `Ok(())`
    /// let result = MyInstance::spec()
    ///     .coolness(47)
    ///     .build()
    ///     .unwrap()
    ///     .spawn()
    ///     .await;
    /// assert_eq!(Ok(MyInstance { coolness: 47 }), result);
    /// # });
    /// # Ok::<(), SpawnError>(())
    /// ```
    async fn spawn(&self) -> result::Result<Self::Instance, Self::Error>;
}

/// A type which implements the [Builder pattern] and builds a [`Spec`].
///
/// [Builder pattern]:
/// https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder
pub trait SpecBuilder: Default {
    /// The type implementing [`Spec`] which this builder builds.
    type Spec: Spec;
    /// Error type returned for errors when calling member methods.
    type Error;

    /// ```rust
    /// use std::error::Error;
    /// use deadpool_cyclone::instance::{Instance, Spec, SpecBuilder};
    /// # #[derive(Default)]
    /// # struct MyInstance;
    /// # #[async_trait::async_trait]
    /// # impl Instance for MyInstance {
    /// #     type SpecBuilder = Builder;
    /// #     type Error = BuildError;
    /// #     async fn ensure_healthy(&mut self) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn terminate(mut self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # #[async_trait::async_trait]
    /// # impl Spec for MySpec {
    /// #     type Instance = MyInstance;
    /// #     type Error = BuildError;
    /// #     async fn spawn(&self) -> Result<Self::Instance, Self::Error> { Ok(Self::Instance {}) }
    /// # }
    ///
    /// #[derive(Debug, thiserror::Error, PartialEq)]
    /// #[error("failed to build")]
    /// struct BuildError;
    ///
    /// #[derive(Default)]
    /// struct Builder {
    ///     coolness: usize
    /// }
    ///
    /// impl SpecBuilder for Builder {
    ///       type Spec = MySpec;
    ///       type Error = BuildError;
    ///
    ///       fn build(&self) -> Result<Self::Spec, Self::Error> {
    ///           Ok(Self::Spec { coolness: self.coolness })
    ///       }
    /// }
    ///
    /// impl Builder {
    ///     fn coolness(&mut self, c: usize) -> &mut Self {
    ///         self.coolness = c;
    ///         self
    ///     }
    /// }
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct MySpec { coolness: usize }
    ///
    /// # tokio_test::block_on(async {
    /// // A successful build returns the `Spec`
    /// let result = Builder::default()
    ///     .coolness(47)
    ///     .build();
    /// assert_eq!(Ok(MySpec { coolness: 47 }), result);
    /// # });
    /// # Ok::<(), BuildError>(())
    /// ```
    fn build(&self) -> result::Result<Self::Spec, Self::Error>;
}

/// Represents a generic instance of a managed resource.
///
/// An Instance can be spawned, queried for its health, and terminated all in a consistent and
/// controlled manner.
#[async_trait]
pub trait Instance {
    /// A type that implements the [Builder pattern] to build a type implementing [`Spec`].
    ///
    /// [Builder pattern]:
    /// https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder
    type SpecBuilder: SpecBuilder + Default;
    /// Error type returned for errors when calling member methods.
    type Error;

    /// Returns a default [`Self::SpecBuilder`] that will build an Instance.
    #[must_use]
    fn spec() -> Self::SpecBuilder {
        Self::SpecBuilder::default()
    }

    /// Returns `()` if instance is healthy, and a [`Self::Error`] if unhealthy.
    ///
    /// Callers can use match destructuring to determine the type or cause of the unhealthiness.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::error::Error;
    /// use deadpool_cyclone::instance::{Instance, Spec, SpecBuilder};
    /// # struct MySpec { healthy: bool }
    /// # #[async_trait::async_trait]
    /// # impl Spec for MySpec {
    /// #       type Instance = MyInstance;
    /// #       type Error = Unhealthy;
    /// #       async fn spawn(&self) -> Result<Self::Instance, Self::Error> {
    /// #           Ok(Self::Instance { healthy: self.healthy })
    /// #       }
    /// # }
    /// # #[derive(Default)]
    /// # struct Builder { healthy: bool };
    /// # impl SpecBuilder for Builder {
    /// #       type Spec = MySpec;
    /// #       type Error = Box<dyn Error + 'static>;
    /// #       fn build(&self) -> Result<Self::Spec, Self::Error> {
    /// #           Ok(Self::Spec { healthy: self.healthy })
    /// #       }
    /// # }
    /// # impl Builder {
    /// #       fn healthy(&mut self, healthy: bool) -> &mut Self {
    /// #           self.healthy = healthy;
    /// #           self
    /// #       }
    /// # }
    ///
    /// #[derive(Debug, thiserror::Error, PartialEq)]
    /// #[error("not healthy")]
    /// struct Unhealthy;
    ///
    /// struct MyInstance { healthy: bool }
    ///
    /// #[async_trait::async_trait]
    /// impl Instance for MyInstance {
    ///     type SpecBuilder = Builder;
    ///     type Error = Unhealthy;
    ///
    ///     async fn ensure_healthy(&mut self) -> Result<(), Self::Error> {
    ///         if self.healthy {
    ///             Ok(())
    ///         } else {
    ///             Err(Unhealthy)
    ///         }
    ///     }
    ///
    ///     async fn terminate(mut self) -> Result<(), Self::Error> { Ok(()) }
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// // Healthy instances return `Ok(())`
    /// let mut healthy_instance = MyInstance::spec()
    ///     .healthy(true)
    ///     .build()
    ///     .unwrap()
    ///     .spawn()
    ///     .await
    ///     .unwrap();
    /// let result = healthy_instance.ensure_healthy().await;
    /// assert_eq!(Ok(()), result);
    ///
    /// // Unhealthy instances return an `Err` which can be destructured to check what might be
    /// // unhealthy
    /// let mut unhealthy_instance = MyInstance::spec()
    ///     .healthy(false)
    ///     .build()
    ///     .unwrap()
    ///     .spawn()
    ///     .await
    ///     .unwrap();
    /// let result = unhealthy_instance.ensure_healthy().await;
    /// assert_eq!(Err((Unhealthy)), result);
    /// # });
    /// # Ok::<(), Unhealthy>(())
    /// ```
    async fn ensure_healthy(&mut self) -> result::Result<(), Self::Error>;

    /// Terminates the instance and returns `()` on success or a [`Self::Error`] on failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::error::Error;
    /// use deadpool_cyclone::instance::{Instance, Spec, SpecBuilder};
    /// # struct MySpec;
    /// # #[async_trait::async_trait]
    /// # impl Spec for MySpec {
    /// #       type Instance = MyInstance;
    /// #       type Error = TerminationError;
    /// #       async fn spawn(&self) -> Result<Self::Instance, Self::Error> {
    /// #           Ok(Self::Instance {})
    /// #       }
    /// # }
    /// # #[derive(Default)]
    /// # struct Builder;
    /// # impl SpecBuilder for Builder {
    /// #       type Spec = MySpec;
    /// #       type Error = Box<dyn Error + 'static>;
    /// #       fn build(&self) -> Result<Self::Spec, Self::Error> { Ok(Self::Spec {}) }
    /// # }
    ///
    /// #[derive(Debug, thiserror::Error, PartialEq)]
    /// #[error("failed to terminate")]
    /// struct TerminationError;
    ///
    /// struct MyInstance;
    ///
    /// #[async_trait::async_trait]
    /// impl Instance for MyInstance {
    ///     type SpecBuilder = Builder;
    ///     type Error = TerminationError;
    ///
    ///     async fn ensure_healthy(&mut self) -> Result<(), Self::Error> { Ok(()) }
    ///
    ///     async fn terminate(mut self) -> Result<(), Self::Error> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// // A successful termination returns `Ok(())`
    /// let mut instance = MyInstance::spec()
    ///     .build()
    ///     .unwrap()
    ///     .spawn()
    ///     .await
    ///     .unwrap();
    /// let result = instance.terminate().await;
    /// assert_eq!(Ok(()), result);
    /// # });
    /// # Ok::<(), TerminationError>(())
    /// ```
    async fn terminate(&mut self) -> result::Result<(), Self::Error>;

    /// Get the id of the underlying child runtime
    fn id(&self) -> u32;
}

// async fn spawn<B, E, I, S>(builder: &B) -> Result<impl Instance<Error = E>, E>
// where
//     B: InstanceSpecBuilder<Spec = S, Error = E>,
//     S: InstanceSpec<Error = E, Instance = I>,
//     I: Instance<SpecBuilder = B, Error = E>,
// {
//     builder.build()?.spawn().await
// }
