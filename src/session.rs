use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinSet,
};
use tokio::{
    time::{sleep, Duration},
};
use tokio_util::sync::CancellationToken;
pub mod state;

pub struct SessionActor {
    broadcast_send: broadcast::Sender<state::Response>,
    mpsc_recv: mpsc::Receiver<state::Request>,
    session_state: i32,
}

impl SessionActor {
    fn new(websocket_source: mpsc::Receiver<WebSocket>) -> Self {
        let (broadcast_sender, _broadcast_recv) = broadcast::channel(32);
        let broadcast_send = broadcast_sender.clone();
        let (mpsc_send, mpsc_recv) = mpsc::channel(32);
        let _new_conn_handle = tokio::spawn(async move {
            websocket_handler(websocket_source, broadcast_sender, mpsc_send).await;
        });

        Self {
            broadcast_send,
            mpsc_recv,
            session_state: 0,
        }
    }

    async fn run(&mut self) {
        let mut result; // Default vale
        while let Some(request) = self.mpsc_recv.recv().await {
            match request {
                state::Request::Increment() => self.session_state += 1,
                state::Request::Decrement() => self.session_state -= 1,
                state::Request::Reset() => self.session_state = 0,
            }
            result = self
                .broadcast_send
                .send(state::Response::Value(self.session_state));
            if result.is_err() {
                break;
            }
        }
    }
}

async fn websocket_handler(
    mut websocket_source: mpsc::Receiver<WebSocket>,
    broadcast_sender: broadcast::Sender<state::Response>,
    mpsc_send: mpsc::Sender<state::Request>,
) {
    let mut tasks = JoinSet::<()>::new();
    // Make sure we have at least one websocket in the task set
    if let Some(new_ws) = websocket_source.recv().await {
        build_ws_tasks(new_ws, &broadcast_sender, mpsc_send.clone(), &mut tasks)
    } else {
        return;
    }
    loop {
        tokio::select! {
            recv_ws = websocket_source.recv() => {
                if let Some(new_ws) = recv_ws{
                    build_ws_tasks(new_ws, &broadcast_sender, mpsc_send.clone(), &mut tasks)
                } else {break}
            },
            // Break out of loop if we run out of tasks or get a join error
            taskres = tasks.join_next() => {
                if let Some(task_return) = taskres {
                    if task_return.is_err() {
                        break
                    }
                } else {
                    break
                }
            },
        }
    }
}

fn build_ws_tasks(
    new_ws: WebSocket,
    broadcast_sender: &broadcast::Sender<state::Response>,
    mpsc_send: mpsc::Sender<state::Request>,
    join_set: &mut JoinSet<()>,
) {
    let (ws_sender, ws_receiver) = new_ws.split();
    let bc_reciever = broadcast_sender.subscribe();
    let token = CancellationToken::new();
    let token_2 = token.clone();

    join_set.spawn(async move { broadcast_to_websocket(bc_reciever, ws_sender, token).await });
    join_set.spawn(async move { websocket_to_mpsc(ws_receiver, mpsc_send, token_2).await });
}

async fn websocket_to_mpsc(
    mut ws_receiver: SplitStream<WebSocket>,
    mpsc_send: mpsc::Sender<state::Request>,
    cancel_token: CancellationToken,
) {
    loop {
        let timeout = sleep(Duration::from_secs(60 * 15)); // Kill if inactive for 15 mins
        tokio::pin!(timeout);
        tokio::select! {
            recv = ws_receiver.next() => {
                if let Some(result) = recv {
                    if let Ok(Message::Text(s)) = result {
                            let parsed_msg = serde_json::from_str(&s).unwrap();
                            if mpsc_send.send(parsed_msg).await.is_err() {
                                break
                            }
                    }
                } else {
                    break // Break if ws_receiver closes
                }
            },
            _ = timeout => break,
            _ = cancel_token.cancelled() => break,
            else => break,
        }
    }
    cancel_token.cancel();
}

async fn broadcast_to_websocket(
    mut bc_reciever: broadcast::Receiver<state::Response>,
    mut ws_sender: SplitSink<WebSocket, Message>,
    cancel_token: CancellationToken,
) {
    let mut ws_send_result;
    loop {
        tokio::select! {
            Ok(x) = bc_reciever.recv() => {
                let ws_message = match x {
                    state::Response::Close() => Message::Close(None),
                    state::Response::Value(x) => Message::Text(format!("{}",x)),
                };
                ws_send_result = ws_sender.send(ws_message).await;
            },
            _ = cancel_token.cancelled() => break,
            else => {cancel_token.cancel(); break},
        }
        if ws_send_result.is_err() {
            cancel_token.cancel();
            break;
        }
    }
}
#[derive(Debug, Clone)]
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
