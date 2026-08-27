use actix_web::{HttpRequest, HttpResponse, get, rt, web};
use actix_ws::AggregatedMessage;
use function_name::named;
use scoretracker::{debug, info, log_fn_name, log_should_print_debug, util::byte_count::ByteCount};
use smol::{lock::Mutex, stream::StreamExt};
use std::{sync::Arc, thread, time::Duration};

#[get("/api/worker_connect")]
#[named]
pub async fn worker_connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    log_fn_name!(auto);
    info!("received worker_connect request");

    let (res, session_owned, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(ByteCount::mebibytes(1.0).as_usize());

    let session_mutex = Arc::new(Mutex::new(session_owned));

    // start task but don't wait for it
    // Receiver thread
    let session = Arc::clone(&session_mutex);
    rt::spawn(async move {
        log_fn_name!("worker_connect:receiver");
        log_should_print_debug!(true);
        debug!("receiver started");
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            debug!("got message: {msg:?}");
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    // echo text message
                    debug!("text message received: {text}");
                    session.lock().await.text(format!("i heard: {text}")).await.unwrap();
                    debug!("response sent");
                }

                Ok(AggregatedMessage::Binary(bin)) => {
                    // echo binary message
                    debug!("binary message received: len={}", bin.len());
                    session.lock().await.binary(bin).await.unwrap();
                    debug!("response sent");
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    // respond to PING frame with PONG frame
                    debug!("ping message received: len={}", msg.len());
                    session.lock().await.pong(&msg).await.unwrap();
                    debug!("response sent");
                }

                _ => {}
            }
        }
        debug!("receiver loop finished");
    });

    // Sender thread
    if true {
        let session = Arc::clone(&session_mutex);
        rt::spawn(async move {
            log_fn_name!("worker_connect:sender");
            log_should_print_debug!(true);
            // Send some random messages for testing
            for i in 1..=10 {
                rt::time::sleep(Duration::from_secs(i)).await;
                let message = format!("Hello this is an automatic message no. {i}");
                debug!("sending message: '{message}'");
                {
                    let mut lock = session.lock().await;
                    lock.text(message).await.unwrap();
                }
                debug!("message sent");
            }
        });
    }

    // respond immediately with response connected to WS session
    Ok(res)
}
