use std::time::{Duration, Instant};

use actix_web::web;
use actix_ws::{CloseCode, CloseReason, Message, ProtocolError};


use base64::Engine;
use futures_util::StreamExt;
use tokio_stream::wrappers::IntervalStream;

use crate::{
    handlers::{self, AppError},
    request, user_message_service, AppData,
};

/// How often heartbeat pings are sent.
///
/// Should be half (or less) of the acceptable client timeout.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn submit_user_message_ws(
    data: web::Data<AppData>,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    query: web::Query<request::UserMessageSubmitProps>,
) {
    let mut audio_data: Vec<u8> = vec![];

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
                        last_heartbeat = Instant::now();
                        audio_data.extend(data);
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
        let _ = user_message_service::add(&mut *conn, 1, 1, audio_data)
            .await
            .map_err(handlers::report_postgres_err);
    }

    // attempt to close connection gracefully
    let _ = session.close(reason).await;
}

const BLOCK_INTERVAL: Duration = Duration::from_millis(10);
const BLOCK_SIZE: usize = 1024;

// feed the message in 1kb blocks at a slightly rearpace
pub async fn receive_user_message_ws(
    data: web::Data<AppData>,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    query: web::Query<request::UserMessageReceiveProps>,
) {
    // open db connection
    let val = match data.pool.get().await {
        Ok(mut obj) => {
            let conn: &mut tokio_postgres::Client = &mut *obj;
            match user_message_service::get_by_user_message_id(&mut *conn, query.user_message_id)
                .await
            {
                Ok(Some(v)) => Ok(v),
                Ok(None) => Err(AppError::NotFound),
                Err(e) => Err(handlers::report_postgres_err(e)),
            }
        }
        Err(e) => Err(handlers::report_pool_err(e)),
    };

    let audio_data = match val {
        Ok(a) => a.audio_data,
        Err(e) => {
            let _ = session
                .close(Some(CloseReason {
                    code: CloseCode::Error,
                    description: Some(e.to_string()),
                }))
                .await;
            return;
        }
    };

    let mut audio_data = audio_data.chunks(BLOCK_SIZE);

    enum TaskUpdateKind {
        // we received a message from the client
        ClientMessage(Result<Message, ProtocolError>),
        // we need to
        NeedToSendData,
    }

    let heartbeat_stream = IntervalStream::new(tokio::time::interval(BLOCK_INTERVAL))
        .map(|_| TaskUpdateKind::NeedToSendData);

    let client_message_stream = msg_stream.map(|x| TaskUpdateKind::ClientMessage(x));

    let mut joint_stream = futures_util::stream_select!(heartbeat_stream, client_message_stream,);

    let reason = loop {
        match joint_stream.next().await.unwrap() {
            // received message from WebSocket client
            TaskUpdateKind::ClientMessage(Ok(msg)) => {
                log::debug!("msg: {msg:?}");
                match msg {
                    Message::Continuation(_) | Message::Binary(_) | Message::Text(_) => {
                        break Some(CloseReason {
                            code: CloseCode::Unsupported,
                            description: None,
                        });
                    }
                    Message::Close(_) => break None,
                    Message::Ping(bytes) => {
                        let _ = session.pong(&bytes).await;
                    }
                    // no-op; ignore
                    Message::Pong(_) | Message::Nop => {}
                };
            }
            // client WebSocket stream error
            TaskUpdateKind::ClientMessage(Err(err)) => {
                log::error!("{}", err);
                break None;
            }
            // heartbeat interval ticked
            TaskUpdateKind::NeedToSendData => {
                if let Some(chunk) = audio_data.next() {
                    let chunk: Vec<u8> = chunk.into();
                    let _ = session.binary(chunk).await;
                } else {
                    break None;
                }
            }
        }
    };

    // attempt to close connection gracefully
    let _ = session.close(reason).await;
}
