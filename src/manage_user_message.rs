use std::time::{Duration, Instant};

use actix_web::web;
use actix_ws::{CloseCode, CloseReason, Message, ProtocolError};

use base64::Engine;
use futures_util::StreamExt;
use tokio_stream::wrappers::IntervalStream;

use crate::{handlers, user_message_service, AppData};

/// How often heartbeat pings are sent.
///
/// Should be half (or less) of the acceptable client timeout.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn manage_user_message_ws(
    data: web::Data<AppData>,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let mut last_heartbeat = Instant::now();

    enum TaskUpdateKind {
        // we need to send a heartbeat
        NeedToSendHeartbeat,
        // we received a message from the client
        ClientMessage(Result<Message, ProtocolError>),
    }

    let heartbeat_stream = IntervalStream::new(tokio::time::interval(HEARTBEAT_INTERVAL))
        .map(|_| TaskUpdateKind::NeedToSendHeartbeat);
    let client_message_stream = msg_stream.map(|x| TaskUpdateKind::ClientMessage(x));

    let mut joint_stream = futures_util::stream_select!(heartbeat_stream, client_message_stream,);

    let mut audio_data_raw: Vec<u8> = vec![];

    let reason = loop {
        match joint_stream.next().await.unwrap() {
            // received message from WebSocket client
            TaskUpdateKind::ClientMessage(Ok(msg)) => {
                log::debug!("msg: {msg:?}");
                match msg {
                    Message::Text(_) => {
                        break Some(CloseReason {
                            code: CloseCode::Unsupported,
                            description: Some(String::from("Only binary data supported")),
                        });
                    }
                    Message::Binary(data) => {
                        audio_data_raw.extend(data);
                    }
                    Message::Close(_) => break None,
                    Message::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        let _ = session.pong(&bytes).await;
                    }
                    Message::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }
                    Message::Continuation(_) => {
                        break Some(CloseReason {
                            code: CloseCode::Unsupported,
                            description: Some(String::from("No support for continuation frame.")),
                        });
                    }
                    // no-op; ignore
                    Message::Nop => {}
                };
            }
            // client WebSocket stream error
            TaskUpdateKind::ClientMessage(Err(err)) => {
                log::error!("{}", err);
                break None;
            }
            // heartbeat interval ticked
            TaskUpdateKind::NeedToSendHeartbeat => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    log::info!("client has not sent heartbeat in over {CLIENT_TIMEOUT:?}");
                    break Some(CloseReason {
                        code: CloseCode::Protocol,
                        description: Some(String::from("server: timed out")),
                    });
                }
                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        }
    };

    // open db connection
    if let Ok(mut obj) = data.pool.get().await.map_err(handlers::report_pool_err) {
        let conn: &mut tokio_postgres::Client = &mut *obj;
        let audio_data = base64::engine::general_purpose::STANDARD_NO_PAD.encode(&audio_data_raw);
        let _ = user_message_service::add(&mut *conn, 1, 1, audio_data)
            .await
            .map_err(handlers::report_postgres_err);
    }

    // attempt to close connection gracefully
    let _ = session.close(reason).await;
}
