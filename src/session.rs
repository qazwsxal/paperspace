use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
};

use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
pub mod websocket;

pub struct SessionActor {
    broadcast_send: broadcast::Sender<websocket::Response>,
    mpsc_recv: mpsc::Receiver<websocket::Request>,
    session_state: i32,
}

impl SessionActor {
    fn new(mut websocket_source: mpsc::Receiver<WebSocket>) -> Self {
        let (broadcast_sender, _broadcast_recv) = broadcast::channel(32);
        let broadcast_send = broadcast_sender.clone();
        let (mpsc_send, mpsc_recv) = mpsc::channel(32);
        let new_conn_handle = tokio::spawn(async move {
            while let Some(new_ws) = websocket_source.recv().await {
                build_ws_tasks(new_ws, &broadcast_sender, mpsc_send.clone())
            }
        });

        Self {
            broadcast_send,
            mpsc_recv,
            session_state: 0,
        }
    }

    async fn run(&mut self) {
        while let Some(request) = self.mpsc_recv.recv().await {
            let mut result = Ok(0); // Default vale
            match request {
                websocket::Request::Increment() => self.session_state += 1,
                websocket::Request::Decrement() => self.session_state -= 1,
                websocket::Request::Get() => {
                    result = self
                        .broadcast_send
                        .send(websocket::Response::Value(self.session_state));
                }
            };
            if result.is_err() {
                break;
            }
        }
    }
}

fn build_ws_tasks(
    new_ws: WebSocket,
    broadcast_sender: &broadcast::Sender<websocket::Response>,
    mpsc_send: mpsc::Sender<websocket::Request>,
) {
    let (ws_sender, ws_receiver) = new_ws.split();
    let bc_reciever = broadcast_sender.subscribe();
    tokio::spawn(async move { broadcast_to_websocket(bc_reciever, ws_sender).await });
    tokio::spawn(async move { websocket_to_mpsc(ws_receiver, mpsc_send).await });
}

async fn websocket_to_mpsc(
    mut ws_receiver: SplitStream<WebSocket>,
    mpsc_send: mpsc::Sender<websocket::Request>,
) -> Result<(), mpsc::error::SendError<websocket::Request>> {
    let mut mpsc_send_result = Ok(());
    loop {
        if let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Text(s) = msg {
                mpsc_send_result = mpsc_send.send(serde_json::from_str(&s).unwrap()).await
            }
        } else {
            break mpsc_send_result;
        }
    }
}

async fn broadcast_to_websocket(
    mut bc_reciever: broadcast::Receiver<websocket::Response>,
    mut ws_sender: SplitSink<WebSocket, Message>,
) -> Result<(), axum::Error> {
    let mut ws_send_result = Ok(());
    loop {
        if let Ok(x) = bc_reciever.recv().await {
            match x {
                websocket::Response::Close() => {
                    ws_send_result = ws_sender.send(Message::Close(None)).await
                }
                websocket::Response::Value(x) => {
                    ws_send_result = ws_sender.send(Message::Text(format!("{}", x))).await
                }
            }
        } else {
            break ws_send_result;
        }
        if let Err(_) = ws_send_result {
            break ws_send_result;
        }
    }
}
pub struct SessionActorHandle {
    pub sender: mpsc::Sender<WebSocket>,
}
impl SessionActorHandle {
    pub async fn new() -> SessionActorHandle {
        let (sender, receiver) = mpsc::channel(32);
        let mut actor = SessionActor::new(receiver);
        tokio::spawn(async move { actor.run().await });
        Self { sender }
    }
}
