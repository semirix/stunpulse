use std::env;

use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use tokio_postgres::NoTls;

#[derive(Default)]
pub struct TestContextBuilder {
    seeds: Vec<String>,
}

impl TestContextBuilder {
    pub fn with_seed(mut self, seed: &str) -> TestContextBuilder {
        self.seeds.push(seed.to_owned());
        self
    }

    pub async fn instantiate(self) -> TestContext {
        let postgres = Postgres::default().start().await.unwrap();
        let uri = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            postgres.get_host().await.unwrap(),
            postgres.get_host_port_ipv4(5432).await.unwrap()
        );
        let (client, connection) = tokio_postgres::connect(&uri, NoTls).await.unwrap();

        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("connection error: {}", error);
            }
        });

        for seed in &self.seeds {
            client.batch_execute(seed).await.unwrap();
        }

        env::set_var("DATABASE_URL", &uri);

        TestContext { postgres }
    }
}

pub struct TestContext {
    postgres: ContainerAsync<Postgres>,
}

impl TestContext {
    pub fn build() -> TestContextBuilder {
        TestContextBuilder::default()
    }

    pub async fn shutdown(self) {
        self.postgres.stop().await.unwrap();
        self.postgres.rm().await.unwrap();
    }
}
