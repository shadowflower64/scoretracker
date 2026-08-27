use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::{AggregatedMessage, AggregatedMessageStream, Session};
use function_name::named;
use scoretracker::hive::worker::ws::{ClientboundMessage, ServerboundMessage};
use scoretracker::log_fn_name;
use scoretracker::log_should_print_debug;
use scoretracker::util::byte_count::ByteCount;
use scoretracker::{debug, info};
use smol::{lock::Mutex, stream::StreamExt};
use std::{sync::Arc, time::Duration};

// TODO: this entire file - add error handling

pub const WORKER_CONNECTION_DEBUG: bool = true;
pub const WORKER_CONNECTION_MESSAGE_DEBUG: bool = true;
pub const WORKER_CONNECTION_MESSAGE_RAW_DEBUG: bool = false;

#[derive(Clone)]
pub struct SmallWorkerClientHandle {
    pub session: Arc<Mutex<Session>>,
    pub communication_type: CommunicationType,
}

impl SmallWorkerClientHandle {
    pub fn later(&self, task: impl AsyncFn(SmallWorkerClientHandle) + 'static) {
        let client = self.clone();
        rt::spawn(async move { task(client).await });
    }

    pub async fn send(&self, msg: ClientboundMessage) {
        send_any(&self.session, self.communication_type, msg).await
    }
}

// Note: this function should not take a long time, as it prevents reading further messages from the client. Use `client.later` to spawn an async task.
// none of the actix_web async function should be blocking,, because the async worker is singlethreaded, blocking prevents all other functions from working.
#[named]
pub async fn handle_message(client: &SmallWorkerClientHandle, msg: ServerboundMessage) {
    log_fn_name!(auto);
    match msg {
        ServerboundMessage::Capabilities(_) => todo!(),
        ServerboundMessage::TestingGotcha { message } => {
            let message = format!("i heard this: {message:?}");
            client.send(ClientboundMessage::TestingIHeard { message }).await
        }
        ServerboundMessage::TestingHello { message } => {
            info!("got hello message from client: {message}");
            client.later(async |client| {
                rt::time::sleep(Duration::from_secs(5)).await;

                let message = format!("5 seconds have passed since the hello message");
                client.send(ClientboundMessage::TestingLater { message }).await;

                rt::time::sleep(Duration::from_secs(5)).await;

                let message = format!("10 seconds have passed since the hello message");
                client.send(ClientboundMessage::TestingLater { message }).await;
            });
        }
    }

    // todo!("handle_message: {msg:?}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationType {
    TextJson,
    BinaryMessagePack,
}

#[named]
pub async fn send_raw_text(session: &Mutex<Session>, text: &str) {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_MESSAGE_RAW_DEBUG);
    debug!("sending text: {text:?}");
    session.lock().await.text(text).await.unwrap(); // TODO: error handling
}

#[named]
pub async fn send_as_json(session: &Mutex<Session>, msg: ClientboundMessage) {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_MESSAGE_DEBUG);
    debug!("sending as json: {msg:?}");
    send_raw_text(session, &serde_json::to_string(&msg).unwrap()).await // TODO: error handling
}

#[named]
pub async fn send_raw_bytes(session: &Mutex<Session>, bytes: Vec<u8>) {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_MESSAGE_RAW_DEBUG);
    debug!("sending bytes: {bytes:?}");
    session.lock().await.binary(bytes).await.unwrap(); // TODO: error handling
}

#[named]
pub async fn send_as_messagepack(session: &Mutex<Session>, msg: ClientboundMessage) {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_MESSAGE_DEBUG);
    debug!("sending as messagepack: {msg:?}");
    send_raw_bytes(session, rmp_serde::to_vec(&msg).unwrap()).await // TODO: error handling
}

pub async fn send_any(session: &Mutex<Session>, communication_type: CommunicationType, msg: ClientboundMessage) {
    match communication_type {
        CommunicationType::TextJson => send_as_json(session, msg).await,
        CommunicationType::BinaryMessagePack => send_as_messagepack(session, msg).await,
    }
}

#[named]
pub async fn worker_connection_receiver(session: Arc<Mutex<Session>>, mut stream: AggregatedMessageStream) {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_DEBUG);

    debug!("receiver started");

    while let Some(raw_msg) = stream.next().await {
        // debug!("got message: {msg:?}");
        match raw_msg {
            Ok(AggregatedMessage::Text(text)) => {
                // Try to deserialize as JSON
                debug!("received text message: {text}");

                let msg: ServerboundMessage = serde_json::from_slice(&text.as_bytes()).unwrap();
                debug!("deserialized message: {msg:?}");

                handle_message(
                    &SmallWorkerClientHandle {
                        session: Arc::clone(&session),
                        communication_type: CommunicationType::TextJson,
                    },
                    msg,
                )
                .await;
            }

            Ok(AggregatedMessage::Binary(bin)) => {
                // Try to deserialize as MessagePack
                debug!("received binary message: len={}: {bin:?}", bin.len());

                let msg: ServerboundMessage = rmp_serde::from_slice(&bin).unwrap();
                debug!("deserialized message: {msg:?}");

                handle_message(
                    &SmallWorkerClientHandle {
                        session: Arc::clone(&session),
                        communication_type: CommunicationType::BinaryMessagePack,
                    },
                    msg,
                )
                .await;
            }

            Ok(AggregatedMessage::Ping(msg)) => {
                debug!("received ping message: len={}", msg.len());
                session.lock().await.pong(&msg).await.unwrap();
                // debug!("response sent");
            }

            _ => {}
        }
    }
    debug!("receiver loop finished");
}

#[named]
pub async fn worker_connection_sender(session: Arc<Mutex<Session>>) {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_DEBUG);
    // Send some random messages for testing
    for i in 1..=10 {
        rt::time::sleep(Duration::from_secs(i)).await;
        let message = format!("Hello this is an automatic message no. {i}");
        send_as_json(&session, ClientboundMessage::TestingAutomated { message }).await
    }
}

#[get("/api/worker_connect")]
#[named]
pub async fn worker_connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_DEBUG);
    debug!("received worker_connect request");

    let (res, session_owned, stream) = actix_ws::handle(&req, stream)?;
    let stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(ByteCount::mebibytes(1.0).as_usize());

    let session_mutex = Arc::new(Mutex::new(session_owned));

    // Spawn receiver thread
    let session = Arc::clone(&session_mutex);
    rt::spawn(async move { worker_connection_receiver(session, stream).await });

    // Spawn sender thread
    let session = Arc::clone(&session_mutex);
    rt::spawn(async move { worker_connection_sender(session).await });

    // Respond immediately with response connected to WS session
    Ok(res)
}
