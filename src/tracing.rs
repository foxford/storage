use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub fn init() -> anyhow::Result<WorkerGuard> {
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());

    let subscriber = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .json()
        .flatten_event(true);

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(subscriber);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(guard)
}
