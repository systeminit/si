// Runnable trait which can be used as a trait object (i.e. `Box<dyn Runnable>`), containing a
// method which moves `self` (i.e. `fn run(self)`).
//
// See: https://users.rust-lang.org/t/need-explanation-on-how-to-avoid-this-move-out-of-a-box-dyn/98734/3
// See: https://quinedot.github.io/rust-learning/dyn-trait-box-impl.html

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait BoxedRunnable {
    async fn try_boxed_run(self: Box<Self>) -> Result<()>;
}

#[async_trait]
pub trait Runnable: BoxedRunnable {
    async fn try_run(self) -> Result<()>;
}

#[async_trait]
impl<T: Runnable + Send> BoxedRunnable for T {
    async fn try_boxed_run(self: Box<Self>) -> Result<()> {
        <Self as Runnable>::try_run(*self).await
    }
}

#[async_trait]
impl Runnable for Box<dyn Runnable + Send + '_> {
    async fn try_run(self) -> Result<()> {
        <dyn Runnable as BoxedRunnable>::try_boxed_run(self).await
    }
}
