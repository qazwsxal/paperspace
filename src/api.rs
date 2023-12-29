use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::mpsc;

use std::collections::HashMap;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::state::PSState;
use crate::{config, session};
use crate::{db, frontend};

pub async fn init(config: config::Config) -> Router {
    Router::new()
        .fallback(frontend::frontend)
        .route("/api/ws", get(ws_handler))
        .with_state(PSState::init(config).await.unwrap())
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(app_state): State<PSState>,
) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    let uuid_entry = params.get("uuid");
    if let Some(uuid) = uuid_entry {
        let uuid_map = app_state.active_sessions.read().await;
        let maybe_handle = uuid_map.get(uuid);
        let socket_sender =
            if let Some(existing_handle) = maybe_handle{
                if existing_handle.sender.is_closed() {
                    drop(uuid_map);
                    init_session(app_state, uuid).await

                } else {
                    existing_handle.sender.clone()
                }
            } else {
                drop(uuid_map);
                init_session(app_state, uuid).await
            };
            ws.on_upgrade(move |socket| infalliable_send(socket_sender, socket))
        } else {
            StatusCode::NOT_FOUND.into_response()
        }
    }

async fn init_session(app_state: PSState, uuid: &str) -> mpsc::Sender<WebSocket> {
    let tx = app_state.pool.begin().await.unwrap();
    let (handle, exit_state) =
        if let Some(session_state) = db::queries::get_session(uuid, tx).await {
            session::SessionActorHandle::load(session_state).await
        } else {
            session::SessionActorHandle::new().await
        };
    let sender = handle.sender.clone();
    app_state
        .active_sessions
        .write().await
        .insert(uuid.to_string(), handle);
    let pool_clone = app_state.pool.clone();
    let uuid_clone = uuid.to_string();
    let map_lock = app_state.active_sessions.clone();
    tokio::spawn(async move {
        let state = exit_state.await.unwrap();
        let mut uuid_map = map_lock.write().await;
        db::queries::save_session(&uuid_clone, state, pool_clone.begin().await.unwrap()).await;
        uuid_map.remove(&uuid_clone)
    });
    sender
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
