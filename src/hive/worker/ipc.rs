use crate::hive::worker::data::{TaskProgress, WorkerData, WorkerStatus};
use crate::{debug, error, info, log_fn_name, log_should_print_debug, warn};
use crossbeam_channel::{Receiver, RecvError};
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::{Arc, LazyLock, Mutex};
use std::thread::JoinHandle;
use std::{process, thread};
use thiserror::Error;

pub type MessageSize = u32;
pub const INCOMING_MESSAGE_SIZE_LIMIT: MessageSize = 1_048_576;
pub const OUTGOING_MESSAGE_SIZE_LIMIT: MessageSize = 1_048_576;
pub const TERMINATION_REQUEST_EXIT_CODE: i32 = 100;
pub const VERBOSE_CONNECTION_HANDLER: bool = true;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read size of the incoming message: {0}")]
    IncomingMessageSizeNotRead(io::Error),
    #[error("incoming message too large: {0} bytes (max {INCOMING_MESSAGE_SIZE_LIMIT} bytes)")]
    IncomingMessageTooLarge(MessageSize),
    #[error("failed to read content of the incoming message: {0}")]
    IncomingMessageContentNotRead(io::Error),
    #[error("failed to deserialize incoming message to json: {0}")]
    IncomingMessageNotDeserialized(serde_json::Error),
    #[error("failed to serialize outgoing message to json: {0}")]
    OutgoingMessageNotSerialized(serde_json::Error),
    #[error("outgoing message too large: {0} bytes (max {OUTGOING_MESSAGE_SIZE_LIMIT} bytes)")]
    OutgoingMessageTooLarge(usize),
    #[error("failed to send size of the outgoing message: {0}")]
    OutgoingMessageSizeNotSent(io::Error),
    #[error("failed to send content of the outgoing message: {0}")]
    OutgoingMessageContentNotSent(io::Error),
    #[error("could not receive from channel: {0}")]
    ReceiveFromChannel(RecvError),
    #[error("could not clone tcp stream: {0}")]
    CloneTcpStream(io::Error),
    #[error("could not fetch peer address: {0}")]
    FetchPeerAddr(io::Error),
    #[error("could not create subscription thread: {0}")]
    CreateSubscriptionThread(io::Error),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subscription {
    WorkerStatus,
    TaskProgress,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingMessage {
    WhoAreYou,
    FetchWorkerStatus,
    Subscribe { subscription: Subscription },
    Unsubscribe { subscription: Subscription },
    TerminationRequest,
}

impl IncomingMessage {
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        log_fn_name!("incoming_message:parse");
        log_should_print_debug!(VERBOSE_CONNECTION_HANDLER);

        let message = serde_json::from_slice(bytes);
        debug!("parsed message: {message:?}");

        message.map_err(Error::IncomingMessageNotDeserialized)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingMessage {
    WhoAreYouResponse { name: String, pid: u32 },
    WorkerStatusResponse { worker_status: WorkerStatus },
}

#[derive(Default)]
pub struct ConnectionInfo {
    pub subscription_worker_status: bool,
    pub subscription_task_progress: bool,
}

fn receive_message_bytes(tcp_stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
    log_fn_name!("receive_message_bytes");
    log_should_print_debug!(VERBOSE_CONNECTION_HANDLER);

    let mut size_bytes = MessageSize::default().to_le_bytes();
    tcp_stream.read_exact(&mut size_bytes).map_err(Error::IncomingMessageSizeNotRead)?;

    let size = MessageSize::from_le_bytes(size_bytes);
    debug!("received size: {size} {size_bytes:?}");
    if size > INCOMING_MESSAGE_SIZE_LIMIT {
        return Err(Error::IncomingMessageTooLarge(size));
    }
    let size = size as usize;

    let mut content = vec![0u8; size];
    tcp_stream.read_exact(&mut content).map_err(Error::IncomingMessageContentNotRead)?;
    debug!("received message content: {content:?}");
    Ok(content)
}

pub fn send_message(tcp_stream: &mut TcpStream, message: &OutgoingMessage) -> Result<(), Error> {
    log_fn_name!("send_message");
    log_should_print_debug!(VERBOSE_CONNECTION_HANDLER);

    debug!("outgoing message: {message:?}");

    let json = serde_json::to_string(message).map_err(Error::OutgoingMessageNotSerialized)?;
    debug!("outgoing message serialized: {json}");

    let content = json.as_bytes();
    debug!("outgoing message content: {content:?}");

    let size = content.len();
    let size: MessageSize = size.try_into().map_err(|_| Error::OutgoingMessageTooLarge(size))?;
    if size > OUTGOING_MESSAGE_SIZE_LIMIT {
        return Err(Error::OutgoingMessageTooLarge(size as usize));
    }

    let size_bytes = size.to_le_bytes();
    debug!("outgoing size: {size} {size_bytes:?}");

    pub const WRITELOCK: LazyLock<Arc<Mutex<()>>> = LazyLock::new(|| Arc::new(Mutex::new(())));
    let arc = Arc::clone(&WRITELOCK);
    {
        let _guard = arc.lock().unwrap();
        tcp_stream.write_all(&size_bytes).map_err(Error::OutgoingMessageSizeNotSent)?;
        tcp_stream.write_all(content).map_err(Error::OutgoingMessageContentNotSent)?;
    }

    debug!("message sent successfully!");
    Ok(())
}

fn handle_incoming_message<MakeWorkerStatusRx: Fn() -> Receiver<WorkerStatus>, MakeTaskProgressRx: Fn() -> Receiver<TaskProgress>>(
    tcp_stream: &mut TcpStream,
    message: IncomingMessage,
    conn: &Arc<Mutex<ConnectionInfo>>,
    _worker_data: &Arc<Mutex<WorkerData>>,
    make_worker_status_rx: MakeWorkerStatusRx,
    _make_task_progress_rx: MakeTaskProgressRx,
) -> Result<(), Error> {
    log_fn_name!("handle_incoming_message");
    log_should_print_debug!(VERBOSE_CONNECTION_HANDLER);

    match message {
        IncomingMessage::WhoAreYou => {
            debug!("responding to 'who are you' message");
            let _ = send_message(
                tcp_stream,
                &OutgoingMessage::WhoAreYouResponse {
                    name: "test name".to_string(), // todo
                    pid: process::id(),
                },
            )
            .inspect_err(|e| error!("failed to send message: {e}; continuing"));
        }
        IncomingMessage::Subscribe {
            subscription: Subscription::WorkerStatus,
        } => {
            let mut conn_guard = conn.lock().unwrap();
            if conn_guard.subscription_worker_status {
                warn!("peer requested to subscribe to worker status but was already subscribed");
            }
            conn_guard.subscription_task_progress = true;
            drop(conn_guard);

            let worker_status_rx = make_worker_status_rx();

            start_subscription_thread(tcp_stream, conn, move |tcp_stream, conn| {
                let worker_status = worker_status_rx.recv().map_err(Error::ReceiveFromChannel)?;
                if conn.lock().unwrap().subscription_worker_status {
                    send_message(tcp_stream, &OutgoingMessage::WorkerStatusResponse { worker_status })?;
                }
                Ok(())
            })?;
        }
        IncomingMessage::Subscribe {
            subscription: Subscription::TaskProgress,
        } => {
            conn.lock().unwrap().subscription_task_progress = true;
        }
        IncomingMessage::Unsubscribe {
            subscription: Subscription::WorkerStatus,
        } => {
            conn.lock().unwrap().subscription_worker_status = false;
        }
        IncomingMessage::Unsubscribe {
            subscription: Subscription::TaskProgress,
        } => {
            conn.lock().unwrap().subscription_task_progress = false;
        }
        IncomingMessage::TerminationRequest => {
            info!("received termination request, exiting with code {TERMINATION_REQUEST_EXIT_CODE}");
            process::exit(TERMINATION_REQUEST_EXIT_CODE);
        }
        a => todo!("not done yet: {a:?}"),
    }
    Ok(())
}

fn recv_connection_loop<MakeWorkerStatusRx: Fn() -> Receiver<WorkerStatus>, MakeTaskProgressRx: Fn() -> Receiver<TaskProgress>>(
    tcp_stream: &mut TcpStream,
    connection_info: &Arc<Mutex<ConnectionInfo>>,
    worker_data: &Arc<Mutex<WorkerData>>,
    make_worker_status_rx: MakeWorkerStatusRx,
    make_task_progress_rx: MakeTaskProgressRx,
) -> Result<(), Error> {
    log_fn_name!("recv_connection_loop");

    let message_bytes = receive_message_bytes(tcp_stream)?;
    match IncomingMessage::parse(&message_bytes) {
        Ok(message) => handle_incoming_message(
            tcp_stream,
            message,
            connection_info,
            &worker_data,
            make_worker_status_rx,
            make_task_progress_rx,
        )?,
        Err(e) => {
            let message_bytes_as_string = String::from_utf8_lossy(&message_bytes);
            error!("could not recognize received message: {e} - received message was: {message_bytes_as_string}; continuing");
        }
    }
    Ok(())
}

fn start_subscription_thread<F: Send + 'static + Fn(&mut TcpStream, &Arc<Mutex<ConnectionInfo>>) -> Result<(), Error>>(
    tcp_stream: &mut TcpStream,
    connection_info: &Arc<Mutex<ConnectionInfo>>,
    loop_fn: F,
) -> Result<JoinHandle<()>, Error> {
    // log_fn_name!("start_connsend_thread");

    let mut tcp_stream = tcp_stream.try_clone().map_err(Error::CloneTcpStream)?;
    let peer_addr = tcp_stream.peer_addr().map_err(Error::FetchPeerAddr)?;
    let connection_info = Arc::clone(&connection_info);

    thread::Builder::new()
        .name(format!("worker:connsend:{}", peer_addr.port()))
        .spawn(move || {
            log_fn_name!("connection_handler");

            loop {
                if let Err(e) = loop_fn(&mut tcp_stream, &connection_info) {
                    error!("a fatal error occured in the connection: {e}; the connection must be terminated");
                    if let Err(e) = tcp_stream.shutdown(Shutdown::Both) {
                        error!("failed to shutdown connection gracefully: {e}")
                    }
                    info!("shutdown connection with: {peer_addr}");
                    break;
                }
            }
        })
        .map_err(Error::CreateSubscriptionThread)
}

fn start_connection_thread(
    mut tcp_stream: TcpStream,
    peer_addr: SocketAddr,
    worker_data: Arc<Mutex<WorkerData>>,
    worker_status_rx: Arc<Receiver<WorkerStatus>>,
    task_progress_rx: Arc<Receiver<TaskProgress>>,
) {
    log_fn_name!("start_connection_thread");

    let connection_info = Arc::new(Mutex::new(ConnectionInfo::default()));
    if let Err(e) = thread::Builder::new()
        .name(format!("worker:conn:{}", peer_addr.port()))
        .spawn(move || {
            log_fn_name!("connection_handler");
            info!("established connection with: {peer_addr}");

            let make_worker_status_rx = || worker_status_rx.deref().clone();
            let make_task_progress_rx = || task_progress_rx.deref().clone();

            loop {
                if let Err(e) = recv_connection_loop(
                    &mut tcp_stream,
                    &connection_info,
                    &worker_data,
                    make_worker_status_rx,
                    make_task_progress_rx,
                ) {
                    error!("a fatal error occured in the connection: {e}; the connection must be terminated");
                    if let Err(e) = tcp_stream.shutdown(Shutdown::Both) {
                        error!("failed to shutdown connection gracefully: {e}")
                    }
                    info!("shutdown connection with: {peer_addr}");
                    break;
                }
            }
        })
    {
        error!("failed to create connection handler thread: {e}");
        return;
    }
}

pub fn start_listener_thread(
    listener: TcpListener,
    worker_data: Arc<Mutex<WorkerData>>,
    worker_status_rx: Receiver<WorkerStatus>,
    task_progress_rx: Receiver<TaskProgress>,
) {
    log_fn_name!("listener");

    let local_address = match listener.local_addr() {
        Ok(addr) => addr,
        Err(e) => {
            error!("failed to get local address of tcp listener: {e}");
            return;
        }
    };

    let worker_status_rx = Arc::new(worker_status_rx);
    let task_progress_rx = Arc::new(task_progress_rx);

    if let Err(e) = thread::Builder::new().name("worker:tcp_listener".to_string()).spawn(move || {
        info!("start listening on {local_address}");

        loop {
            match listener.accept() {
                Ok((tcp_stream, peer_addr)) => {
                    start_connection_thread(
                        tcp_stream,
                        peer_addr,
                        worker_data.clone(),
                        worker_status_rx.clone(),
                        task_progress_rx.clone(),
                    );
                }
                Err(e) => {
                    error!("failed to establish connection with remote peer: {e}");
                }
            }
        }
    }) {
        error!("failed to create listener thread: {e}");
        return;
    }
}
