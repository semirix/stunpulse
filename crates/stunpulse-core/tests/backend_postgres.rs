use futures::future::FutureExt;
use stunpulse_core::{
    backend::{postgres::PostgresBackend, traits::QueueBackend},
    config::Configuration,
    context::{Context, ModuleIdentifier},
};
use testing_utilities::TestContext;

#[tokio::test]
async fn test_initialise() {
    let context = TestContext::build()
        .with_seed(include_str!("seeds/initial.sql"))
        .instantiate()
        .await;
    let backend = PostgresBackend::instantiate(Configuration::default())
        .await
        .expect("Postgres backend didn't instantiate");

    backend
        .validate()
        .await
        .expect("Task table not setup properly");

    context.shutdown().await;
}

#[tokio::test]
async fn test_task_run() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let test_context = TestContext::build()
        .with_seed(include_str!("seeds/initial.sql"))
        .with_seed("INSERT INTO tasks (task_version, task_name, task_parameters, task_metadata) VALUES ('1', 'test', '{}'::JSONB, '{}'::JSONB);")
        .instantiate()
        .await;
    let backend = PostgresBackend::instantiate(Configuration::default())
        .await
        .expect("Postgres backend didn't instantiate");
    let context = Context::new().expect("Couldn't create context");

    backend
        .validate()
        .await
        .expect("Task table not setup properly");

    context
        .load_module(
            ModuleIdentifier {
                version: "1".into(),
                name: "main".into(),
            },
            "../../examples/js/dist/index.wasm".into(),
        )
        .await
        .expect("Couldn't load module");

    let wait = backend
        .task_accept(move |task| {
            let context = context.clone();

            async move { context.run_task(task).await }.boxed()
        })
        .await
        .expect("Couldn't run task");

    assert!(!wait);

    test_context.shutdown().await;
}
