use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use tokio::{net::TcpListener, signal, sync::mpsc};

use std::{collections::HashMap};
use std::{net::SocketAddr};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;
pub mod db;
pub mod frontend;
pub mod session;
pub mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // console_subscriber::init();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "paperspace=debug,tower_http=debug".into()),
        )
    .with(tracing_subscriber::fmt::layer())
    .init();
    let app = Router::new()
        .fallback(frontend::frontend)
        .route("/api/ws", get(ws_handler))
        .with_state(
            state::PSState::init(config::Config::default())
                .await
                .unwrap(),
        )
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000)); // User configurable?
    let listener = TcpListener::bind(addr).await?;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    open::that("http://127.0.0.1:4000/session?uuid=test")?; // Open webUI
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    Query(params): Query<HashMap<String, String>>,
    State(sessions): State<state::Sessions>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    let uuid_entry = params.get("uuid");
    if let Some(uuid) = uuid_entry {
        let get_handle = sessions.read().unwrap().get(uuid).cloned();
        let socket_sender = match get_handle {
            Some(session_handle) if !session_handle.sender.is_closed() => {
                session_handle.sender.clone()
            }
            _ => {
                let handle = session::SessionActorHandle::new().await;
                let sender = handle.sender.clone();
                sessions.write().unwrap().insert(uuid.clone(), handle);
                sender
            }
        };
        ws.on_upgrade(move |socket| infalliable_send(socket_sender, socket))
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
async fn infalliable_send(socket_sender: mpsc::Sender<WebSocket>, mut socket: WebSocket) {
    //send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
    } else {
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    socket_sender.send(socket).await.unwrap()
}
