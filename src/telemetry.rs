use crate::configurations::Configuration;
use tracing::{Level, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber<Sink>(name: &str, debug: bool, sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = if debug {
        "trace".to_string()
    } else {
        "info".to_string()
    };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(
        name.to_string(), // Output the formatted spans to stdout.
        sink,
    );
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

/// Set up basic logging and tracing.
pub fn init_basic_logging() {
    // Initialize logging to console
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Log the version and git commit info
    tracing::info!("Starting: {}", env!("CARGO_PKG_NAME"));
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));
    if let Some(git_hash) = option_env!("GIT_HASH") {
        tracing::info!("Git commit: {}", git_hash);
    }
}

pub fn init_startup_telemetry(configuration: &Configuration) {
    let name = env!("CARGO_PKG_NAME").to_string();
    let subscriber = get_subscriber(&name, configuration.debug, std::io::stdout);
    init_subscriber(subscriber);
}

#[cfg(test)]
pub fn init_test_telemetry() {
    let subscriber = get_subscriber("test", true, std::io::sink);
    init_subscriber(subscriber);
}
