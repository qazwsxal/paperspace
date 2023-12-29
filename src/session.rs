use axum::extract::ws::{Message, WebSocket};
use futures::{
    sink::SinkExt,
    stream::{self, SplitSink, SplitStream,  StreamExt},
};
use tokio::sync::oneshot;
use tokio::{
    sync::broadcast::error::SendError,
    time::{sleep, Duration},
};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

pub mod state;
use state::State;

use self::state::Counter;

pub struct SessionActor {
    broadcast_send: broadcast::Sender<state::Response>,
    mpsc_send: mpsc::Sender<state::Request>,
    mpsc_recv: mpsc::Receiver<state::Request>,
    websocket_source: mpsc::Receiver<WebSocket>,
    state: state::Counter,
    tasks: JoinSet<()>,
}

impl SessionActor {
    fn new(state: state::Counter, websocket_source: mpsc::Receiver<WebSocket>) -> Self {
        let (broadcast_send, _broadcast_recv) = broadcast::channel::<state::Response>(256);
        let (mpsc_send, mpsc_recv) = mpsc::channel(32);
        Self {
            broadcast_send,
            mpsc_send,
            mpsc_recv,
            websocket_source,
            state,
            tasks: JoinSet::<()>::new(),
        }
    }

    async fn run(&mut self) {
        let mut established = false;
        let mut exiting = false;
        loop {
            let timeout = sleep(Duration::from_secs(15));
            tokio::select! {
                // Recieve a message and handle it.
                mpsc_msg = self.mpsc_recv.recv() => {
                    if let Some(request) = mpsc_msg {
                        if self.handle_request(request).await.is_err() {break}
                    } else {
                        break
                    }
                },
                // Set up a new connection.
                opt_new_ws = self.websocket_source.recv() =>  {
                    if let Some(new_ws) = opt_new_ws {
                        self.handle_websocket(new_ws).await;
                        // We have a new connection! so we're established and can't be exiting.
                        established = true;
                        exiting = false;
                    } else {
                        break
                    }
                },
                // If we have no tasks flag us as exiting, if we *then* timeout - shut down the session
                task = self.tasks.join_next(), if established && !exiting => if task.is_none() {exiting = true},
                _ = timeout, if exiting => {break}
            }
        }
        println!("shutdown not handled yet!")
    }

    async fn handle_request(
        &mut self,
        request: state::Request,
    ) -> Result<usize, SendError<state::Response>> {
        let mut responses = self.state.update(request);
        let mut broadcast_result = 0;
        loop {
            if let Some(response) = responses.next().await {
                broadcast_result = self.broadcast_send.send(response)?;
            } else {
                break Ok(broadcast_result);
            }
        }
    }

    async fn handle_websocket(&mut self, new_ws: WebSocket) {
        // session actor pauses during this function so we can't accidentally recieve a message 
        // before sending the full client initialisation stream.
        let bc_reciever = self.broadcast_send.subscribe(); 
        let (mut ws_sender, ws_receiver) = new_ws.split();
        let token = CancellationToken::new();
        let token_2 = token.clone();
        // get a vector of responses that describes the current state.
        let session_init = self.state.dump();
        let mpsc_send = self.mpsc_send.clone();
        self.tasks.spawn(async move {
            // Send messages to catch up client with session
            let _ = ws_sender
                .send_all(
                    &mut stream::iter(session_init)
                        .map(|resp| Ok(Message::Text(serde_json::to_string(&resp).unwrap()))),
                )
                .await;
            // Now handle as normal.
            broadcast_to_websocket(bc_reciever, ws_sender, token).await
        });
        self.tasks
            .spawn(async move { websocket_to_mpsc(ws_receiver, mpsc_send, token_2).await });
    }
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
    println!("closed mpsc")
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
                let ws_message = Message::Text(serde_json::to_string(&x).unwrap());
                ws_send_result = ws_sender.send(ws_message).await;
            },
            _ = cancel_token.cancelled() => break,
            else => {cancel_token.cancel(); break},
        }
        if ws_send_result.is_err() {
            break;
        }
    }
    cancel_token.cancel();
    println!("closed broadcast")
}
#[derive(Debug)]
pub struct SessionActorHandle {
    pub sender: mpsc::Sender<WebSocket>,
}
impl SessionActorHandle {
    pub async fn new() -> (SessionActorHandle, oneshot::Receiver<Counter>) {
        let state = state::Counter::default();
        Self::load(state).await
    }
    pub async fn load(state: state::Counter) -> (SessionActorHandle, oneshot::Receiver<Counter>) {
        let (sender, receiver) = mpsc::channel(32);
        let mut actor = SessionActor::new(state, receiver);
        let (os_send, os_recv) = oneshot::channel();
        tokio::spawn(async move { actor.run().await; os_send.send(actor.state)});
        (Self { sender }, os_recv)
    }
}
