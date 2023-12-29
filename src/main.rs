use tokio::{net::TcpListener, signal};

use std::net::SocketAddr;
// use tower_http::trace::{DefaultMakeSpan, TraceLayer};

// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod api;
pub mod config;
pub mod db;
pub mod frontend;
pub mod session;
pub mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    console_subscriber::init();
    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "paperspace=debug,tower_http=debug".into()),
    //     )
    // .with(tracing_subscriber::fmt::layer())
    // .init();
    let app_config = config::Config::default();
    let app = api::init(app_config).await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000)); // User configurable?
    let listener = TcpListener::bind(addr).await?;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
    });
    let _ = open::that("http://127.0.0.1:4000/session?uuid=test"); // Open webUI, ok if it fails.
    let (_result,) = tokio::join!(server);
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}