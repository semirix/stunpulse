use futures::future::BoxFuture;

use crate::{config::Configuration, error::Error, task::Task};

mod private {
    use crate::backend::postgres::PostgresBackend;

    pub trait Sealed {}

    impl Sealed for PostgresBackend {}
}

#[allow(async_fn_in_trait)]
pub trait QueueBackend: private::Sealed + Send + Sync + Sized + 'static {
    async fn instantiate(configuration: Configuration) -> Result<Self, Error>;
    async fn validate(&self) -> Result<(), Error>;
    async fn task_insert(&self, task: Task) -> Result<(), Error>;
    async fn task_accept(
        &self,
        work: impl Fn(Task) -> BoxFuture<'static, Result<(), Error>> + Send + Sync + 'static,
    ) -> Result<bool, Error>;
}
