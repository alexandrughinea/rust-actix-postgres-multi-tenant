#![allow(async_fn_in_trait)]

use rust_actix_postgres_multi_tenant::configurations::get_configuration;
use rust_actix_postgres_multi_tenant::startup::Application;
use rust_actix_postgres_multi_tenant::telemetry::init_startup_telemetry;
use tracing::event;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configurations!");

    init_startup_telemetry(&configuration);

    let application = Application::build(configuration, None).await?;

    event!(
        tracing::Level::INFO,
        "Listening on http://127.0.0.1:{}/",
        application.port()
    );

    application.run_until_stopped().await?;

    Ok(())
}
