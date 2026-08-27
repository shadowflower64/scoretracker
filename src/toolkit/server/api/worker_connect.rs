use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::{AggregatedMessage, AggregatedMessageStream, Session};
use function_name::named;
use scoretracker::hive::worker::ws::{ClientboundMessage, ServerboundMessage};
use scoretracker::log_fn_name;
use scoretracker::log_should_print_debug;
use scoretracker::util::byte_count::ByteCount;
use scoretracker::{debug, info};
use serde::Deserialize;
use smol::{lock::Mutex, stream::StreamExt};
use std::{sync::Arc, time::Duration};

// TODO: this entire file - add error handling

pub const WORKER_CONNECTION_DEBUG: bool = true;
pub const WORKER_CONNECTION_MESSAGE_DEBUG: bool = true;
pub const WORKER_CONNECTION_MESSAGE_RAW_DEBUG: bool = false;

pub struct WorkerClientHandle {
    pub session: Mutex<Session>,
    pub communication_type: CommunicationType,
}

impl WorkerClientHandle {
    pub fn later(self: &Arc<Self>, task: impl AsyncFn(Arc<WorkerClientHandle>) + 'static) {
        let client = self.clone();
        rt::spawn(async move { task(client).await });
    }

    pub async fn send(&self, msg: ClientboundMessage) {
        match self.communication_type {
            CommunicationType::Json => send_as_json(&self.session, msg).await,
            CommunicationType::MessagePack => send_as_messagepack(&self.session, msg).await,
        }
    }

    pub async fn pong(&self, msg: &[u8]) {
        self.session.lock().await.pong(msg).await.unwrap();
        // debug!("pong response sent");
    }
}

// Note: this function should not take a long time, as it prevents reading further messages from the client. Use `client.later` to spawn an async task.
// none of the actix_web async function should be blocking,, because the async worker is singlethreaded, blocking prevents all other functions from working.
#[named]
pub async fn handle_message(client: &Arc<WorkerClientHandle>, msg: ServerboundMessage) {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationType {
    Json,
    #[default]
    MessagePack,
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

#[named]
pub async fn worker_connection_receiver(client: Arc<WorkerClientHandle>, mut stream: AggregatedMessageStream) {
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

                handle_message(&client, msg).await;
            }
            Ok(AggregatedMessage::Binary(bin)) => {
                // Try to deserialize as MessagePack
                debug!("received binary message: len={}: {bin:?}", bin.len());

                let msg: ServerboundMessage = rmp_serde::from_slice(&bin).unwrap();
                debug!("deserialized message: {msg:?}");

                handle_message(&client, msg).await;
            }
            Ok(AggregatedMessage::Ping(msg)) => {
                debug!("received ping message: len={}: {msg:?}", msg.len());
                client.pong(&msg).await;
            }
            _ => {}
        }
    }
    debug!("receiver loop finished");
}

#[named]
pub async fn worker_connection_sender(client: Arc<WorkerClientHandle>) {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_DEBUG);
    // Send some random messages for testing
    for i in 1..=10 {
        rt::time::sleep(Duration::from_secs(i)).await;
        let message = format!("Hello this is an automatic message no. {i}");
        client.send(ClientboundMessage::TestingAutomated { message }).await
    }
}

#[derive(Debug, Deserialize)]
pub struct WorkerConnectRequest {
    communication_type: Option<CommunicationType>,
}

/// Connect worker to the server via websockets
///
/// You can use the `communication_type` query parameter to choose the format of clientbound messages sent from the server.
/// - `?communication_type=json` will allow you to receive data as JSON.
/// - `?communication_type=message_pack` will allow you to receive data as binary MessagePack.
/// By default, data will be sent via MessagePack.
///
/// Note that both JSON and MessagePack serverbound messages will be accepted by the server in either communication mode.
#[get("/api/worker_connect")]
#[named]
pub async fn worker_connect(
    query: web::Query<WorkerConnectRequest>,
    stream: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    log_fn_name!(auto);
    log_should_print_debug!(WORKER_CONNECTION_DEBUG);
    debug!("received worker_connect request; query: {query:?}");

    let communication_type = query.communication_type.unwrap_or_default();
    let (res, session, stream) = actix_ws::handle(&req, stream)?;
    let stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(ByteCount::mebibytes(1.0).as_usize());

    let client_handle = Arc::new(WorkerClientHandle {
        session: Mutex::new(session),
        communication_type,
    });

    // Spawn receiver thread
    let client = Arc::clone(&client_handle);
    rt::spawn(async move { worker_connection_receiver(client, stream).await });

    // Spawn sender thread
    let client = Arc::clone(&client_handle);
    rt::spawn(async move { worker_connection_sender(client).await });

    // Respond immediately with response connected to WS session
    Ok(res)
}
