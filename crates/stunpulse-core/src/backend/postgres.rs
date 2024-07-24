use std::{env, str::FromStr};

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use futures::future::BoxFuture;
use tokio_postgres::{Config, NoTls};

use crate::{config::Configuration, error::Error, task::Task};

use super::traits::QueueBackend;

pub struct PostgresBackend {
    configuration: Configuration,
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl QueueBackend for PostgresBackend {
    async fn instantiate(configuration: Configuration) -> Result<Self, Error> {
        dotenv::dotenv().ok();

        let config: Config = Config::from_str(&env::var("DATABASE_URL")?)?;
        let manager = PostgresConnectionManager::new(config, NoTls);
        let pool = bb8::Pool::builder()
            .max_size(15)
            .build(manager)
            .await
            .unwrap();

        Ok(Self {
            configuration,
            pool,
        })
    }

    async fn validate(&self) -> Result<(), Error> {
        let connection = self.pool.get_owned().await?;
        let layout = &self.configuration.layout;

        connection
            .query(
                r#"
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name=$1::TEXT
                        AND (column_name, data_type) IN (
                            ($2, 'integer'),
                            ($3, 'text'),
                            ($4, 'text'),
                            ($5, 'jsonb'),
                            ($6, 'jsonb')
                        );
                "#,
                &[
                    &layout.table,
                    &layout.id,
                    &layout.version,
                    &layout.name,
                    &layout.parameters,
                    &layout.metadata,
                ],
            )
            .await?;

        Ok(())
    }

    async fn task_insert(&self, task: Task) -> Result<(), Error> {
        let connection = self.pool.get_owned().await?;
        let layout = &self.configuration.layout;

        connection
            .execute(
                r#"
                    INSERT INTO $1 ($2, $3, $4, $5)
                    VALUES ($6, $7, $8, $9);
                "#,
                &[
                    &layout.table,
                    &layout.version,
                    &layout.name,
                    &layout.parameters,
                    &layout.metadata,
                    &task.version,
                    &task.name,
                    &task.parameters,
                    &task.metadata,
                ],
            )
            .await?;

        Ok(())
    }

    async fn task_accept(
        &self,
        work: impl Fn(Task) -> BoxFuture<'static, Result<(), Error>> + Send + Sync + 'static,
    ) -> Result<bool, Error> {
        let mut connection = self.pool.get_owned().await?;
        let layout = &self.configuration.layout;
        let transaction = connection.transaction().await?;

        let row = transaction
            .query_opt(
                &format!(
                    r#"
                        DELETE FROM {table}
                        WHERE $1 IN (
                            SELECT $1
                            FROM {table}
                            ORDER BY RANDOM()
                            FOR UPDATE
                            SKIP LOCKED
                            LIMIT 1
                        )
                        RETURNING *;
                    "#,
                    table = layout.table
                ),
                &[&layout.id],
            )
            .await?;

        if let Some(row) = row {
            work(Task::from_row_with_layout(row, &layout)).await?;

            transaction.commit().await?;

            Ok(false)
        } else {
            Ok(true)
        }
    }
}
