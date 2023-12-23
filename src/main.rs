use axum::{self, Router};
use open;

use std::net::SocketAddr;

use tokio::{signal, net::TcpListener};

mod frontend;
// mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    console_subscriber::init();

    let app = Router::new().fallback(frontend::frontend);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000)); // User configurable?
    let listener = TcpListener::bind(addr).await?;
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .await.unwrap()
    });
    // open::that("http://127.0.0.1:4000/")?;
    open::that("http://127.0.0.1:4000/")?; // Testing FSN update mechanism
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

