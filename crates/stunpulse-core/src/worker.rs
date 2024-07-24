use std::time::Duration;

use futures::future::FutureExt;
use tokio::time::sleep;

use crate::{backend::traits::QueueBackend, context::Context, error::Error};

pub async fn worker(
    context: Context,
    backend: impl QueueBackend,
    backoff: Duration,
) -> Result<(), Error> {
    loop {
        let context = context.clone();

        let wait = backend
            .task_accept(move |task| {
                let context = context.clone();

                async move { context.run_task(task).await }.boxed()
            })
            .await?;

        if wait {
            sleep(backoff).await;
        }
    }
}
