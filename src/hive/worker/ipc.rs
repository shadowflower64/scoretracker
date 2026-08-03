use crate::hive::worker::Worker;
use crate::hive::worker::status::WorkerStatus;
use crate::{debug, error, info, log_fn_name, log_should_print_debug};
use crossbeam_channel::Receiver;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::io::{self, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{process, thread};
use thiserror::Error;

pub type MessageSize = u32;
pub const INCOMING_MESSAGE_SIZE_LIMIT: MessageSize = 1_048_576;
pub const OUTGOING_MESSAGE_SIZE_LIMIT: MessageSize = 1_048_576;
pub const TERMINATION_REQUEST_EXIT_CODE: i32 = 100;
pub const VERBOSE_CONNECTION_HANDLER: bool = true;

#[derive(Debug, Error)]
pub enum Error {
    #[error("would block")]
    WouldBlock,
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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingMessage {
    WhoAreYou,
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

fn receive_message_bytes(tcp_stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
    log_fn_name!("incoming_message:receive");
    log_should_print_debug!(VERBOSE_CONNECTION_HANDLER);

    let mut size_bytes = MessageSize::default().to_le_bytes();
    tcp_stream.read_exact(&mut size_bytes).map_err(|e| match e.kind() {
        io::ErrorKind::WouldBlock => Error::WouldBlock,
        _ => Error::IncomingMessageSizeNotRead(e),
    })?;

    let size = MessageSize::from_le_bytes(size_bytes);
    debug!("received size: {size} {size_bytes:?}");
    if size > INCOMING_MESSAGE_SIZE_LIMIT {
        return Err(Error::IncomingMessageTooLarge(size));
    }
    let size = size as usize;

    let mut content = vec![0u8; size];
    tcp_stream.read_exact(&mut content).map_err(|e| match e.kind() {
        io::ErrorKind::WouldBlock => Error::WouldBlock, //TODO: this basically requires a state machine to return here... so really this should just use async/await
        _ => Error::IncomingMessageContentNotRead(e),
    })?;
    debug!("received message content: {content:?}");
    Ok(content)
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingMessage {
    WhoAreYouResponse { name: String, pid: u32 },
}

pub fn send_message(message: &OutgoingMessage, tcp_stream: &mut TcpStream) -> Result<(), Error> {
    log_fn_name!("outgoing_message:send");
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

    tcp_stream.write_all(&size_bytes).map_err(Error::OutgoingMessageSizeNotSent)?;
    tcp_stream.write_all(content).map_err(Error::OutgoingMessageContentNotSent)?;
    debug!("message sent successfully!");
    Ok(())
}

fn handle_incoming_message(tcp_stream: &mut TcpStream, message: IncomingMessage, worker_status: Arc<Mutex<WorkerStatus>>) {
    log_fn_name!("worker:handle_message");
    log_should_print_debug!(VERBOSE_CONNECTION_HANDLER);

    match message {
        IncomingMessage::WhoAreYou => {
            debug!("responding to 'who are you' message");
            let _ = send_message(
                &OutgoingMessage::WhoAreYouResponse {
                    name: "test name".to_string(), // todo
                    pid: process::id(),
                },
                tcp_stream,
            )
            .inspect_err(|e| error!("failed to send message: {e}; continuing"));
        }
        IncomingMessage::TerminationRequest => {
            info!("received termination request, exiting with code {TERMINATION_REQUEST_EXIT_CODE}");
            process::exit(TERMINATION_REQUEST_EXIT_CODE);
        }
    }
}

fn connection_loop(tcp_stream: &mut TcpStream, worker_status: Arc<Mutex<WorkerStatus>>) -> Result<(), Error> {
    log_fn_name!("connection_loop");

    match receive_message_bytes(tcp_stream)? {
        Ok(message_bytes) => match IncomingMessage::parse(&message_bytes) {
            Ok(message) => handle_incoming_message(tcp_stream, message, worker_status),
            Err(e) => {
                let message_bytes_as_string = String::from_utf8_lossy(&message_bytes);
                error!("could not recognize received message: {e} - received message was: {message_bytes_as_string}; continuing");
            }
        },
        Err(Error::WouldBlock) => {}
    }
    Ok(())
}

fn start_connection_thread(
    mut tcp_stream: TcpStream,
    peer_addr: SocketAddr,
    worker_status: Arc<Mutex<WorkerStatus>>,
    worker_status_update_rx: Receiver<WorkerStatus>,
) {
    if let Err(e) = tcp_stream.set_nonblocking(true) {
        error!("failed to change tcp stream to nonblocking mode: {e}");
        return;
    }

    if let Err(e) = thread::Builder::new()
        .name(format!("worker:conn:{}", peer_addr.port()))
        .spawn(move || {
            log_fn_name!("connection_handler");
            info!("established connection with: {peer_addr}");

            loop {
                if let Err(e) = connection_loop(&mut tcp_stream, worker_status) {
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
        error!("failed to create handler thread: {e}");
        return;
    }
}

pub fn start_listener_thread(
    listener: TcpListener,
    worker_status: Arc<Mutex<WorkerStatus>>,
    worker_status_update_rx: Receiver<WorkerStatus>,
) {
    let local_address = match listener.local_addr() {
        Ok(addr) => addr,
        Err(e) => {
            error!("failed to get local address of tcp listener: {e}");
            return;
        }
    };
    if let Err(e) = thread::Builder::new().name("worker:tcp_listener".to_string()).spawn(move || {
        log_fn_name!("listener");
        info!("start listening on {local_address}");
        loop {
            match listener.accept() {
                Ok((tcp_stream, peer_addr)) => {
                    start_connection_thread(tcp_stream, peer_addr, worker_status.clone(), worker_status_update_rx.clone());
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
