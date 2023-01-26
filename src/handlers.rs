use super::AppData;

use actix_web::rt;
use actix_web::{
    http::StatusCode, web, Error, HttpRequest, HttpResponse, Responder, ResponseError,
};
use auth_service_api::response::{AuthError, User};
use base64::Engine;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::db_types::SleepEvent;
use crate::db_types::UserMessage;
use crate::response;
use crate::sleep_event_service;
use crate::user_message_service;
use crate::{manage_user_message, request};

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppError {
    DecodeError,
    InternalServerError,
    Unauthorized,
    BadRequest,
    NotFound,
    Unknown,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self)
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::DecodeError => StatusCode::BAD_GATEWAY,
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::BadRequest => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub fn report_postgres_err(e: tokio_postgres::Error) -> AppError {
    log::error!("{}", e);
    AppError::InternalServerError
}

pub fn report_pool_err(e: deadpool_postgres::PoolError) -> AppError {
    log::error!("{}", e);
    AppError::InternalServerError
}

pub fn report_internal_serde_error(e: serde_json::Error) -> AppError {
    log::error!("{}", e);
    AppError::InternalServerError
}

pub fn report_serde_error(e: serde_json::Error) -> AppError {
    log::info!("{}", e);
    AppError::DecodeError
}

pub fn report_base64_err(e: base64::DecodeError) -> AppError {
    log::info!("{}", e);
    AppError::DecodeError
}

pub fn report_auth_err(e: AuthError) -> AppError {
    match e {
        AuthError::ApiKeyNonexistent => AppError::Unauthorized,
        AuthError::ApiKeyUnauthorized => AppError::Unauthorized,
        c => {
            let ae = match c {
                AuthError::InternalServerError => AppError::InternalServerError,
                AuthError::MethodNotAllowed => AppError::InternalServerError,
                AuthError::BadRequest => AppError::InternalServerError,
                AuthError::Network => AppError::InternalServerError,
                _ => AppError::Unknown,
            };
            log::error!("auth: {}", c);
            ae
        }
    }
}

pub async fn get_user_if_api_key_valid(
    auth_service: &auth_service_api::client::AuthService,
    api_key: String,
) -> Result<User, AppError> {
    auth_service
        .get_user_by_api_key_if_valid(api_key)
        .await
        .map_err(report_auth_err)
}

pub fn fill_user_message(x: UserMessage) -> response::UserMessage {
    response::UserMessage {
        creation_time: x.creation_time,
        creator_user_id: x.creator_user_id,
        target_user_id: x.target_user_id,
        audio_data: base64::engine::general_purpose::STANDARD_NO_PAD.encode(&x.audio_data),
    }
}

pub fn fill_sleep_event(x: SleepEvent) -> response::SleepEvent {
    response::SleepEvent {
        creation_time: x.creation_time,
        creator_user_id: x.creator_user_id,
    }
}

// respond with info about stuff
pub async fn info(data: web::Data<AppData>) -> Result<impl Responder, AppError> {
    let info = data.auth_service.info().await.map_err(report_auth_err)?;
    return Ok(web::Json(response::Info {
        service: String::from(super::SERVICE),
        version_major: super::VERSION_MAJOR,
        version_minor: super::VERSION_MINOR,
        version_rev: super::VERSION_REV,
        app_pub_origin: data.app_pub_origin.clone(),
        auth_pub_api_href: info.app_pub_api_href,
        auth_authenticator_href: info.app_authenticator_href,
    }));
}

pub async fn user_message_new(
    req: web::Json<request::UserMessageNewProps>,
    data: web::Data<AppData>,
) -> Result<impl Responder, AppError> {
    // validate api key
    let user = get_user_if_api_key_valid(&data.auth_service, req.api_key.clone()).await?;

    // validate that the other user exists in the first place
    let target_user = data
        .auth_service
        .get_user_by_id(req.target_user_id)
        .await
        .map_err(report_auth_err)?;

    let con: &mut tokio_postgres::Client = &mut *data.pool.get().await.map_err(report_pool_err)?;

    let um = user_message_service::add(
        &mut *con,
        user.user_id,
        target_user.user_id,
        base64::engine::general_purpose::STANDARD_NO_PAD
            .decode(&req.audio_data)
            .map_err(report_base64_err)?,
    )
    .await
    .map_err(report_postgres_err)?;

    return Ok(web::Json(fill_user_message(um)));
}

pub async fn sleep_event_new(
    req: web::Json<request::SleepEventNewProps>,
    data: web::Data<AppData>,
) -> Result<impl Responder, AppError> {
    // validate api key
    let user = get_user_if_api_key_valid(&data.auth_service, req.api_key.clone()).await?;

    let con: &mut tokio_postgres::Client = &mut *data.pool.get().await.map_err(report_pool_err)?;

    let um = sleep_event_service::add(&mut *con, user.user_id)
        .await
        .map_err(report_postgres_err)?;

    return Ok(web::Json(fill_sleep_event(um)));
}

pub async fn user_message_view(
    req: web::Json<request::UserMessageViewProps>,
    data: web::Data<AppData>,
) -> Result<impl Responder, AppError> {
    // api key verification required
    let _ = get_user_if_api_key_valid(&data.auth_service, req.api_key.clone()).await?;

    // get connection
    let con: &mut tokio_postgres::Client = &mut *data.pool.get().await.map_err(report_pool_err)?;

    // get user messages
    let user_messages = user_message_service::query(con, req.into_inner())
        .await
        .map_err(report_postgres_err)?;

    // return
    let mut resp_user_messages = vec![];
    for u in user_messages.into_iter() {
        resp_user_messages.push(fill_user_message(u));
    }

    Ok(web::Json(resp_user_messages))
}

pub async fn sleep_event_view(
    req: web::Json<request::SleepEventViewProps>,
    data: web::Data<AppData>,
) -> Result<impl Responder, AppError> {
    // api key verification required
    let _ = get_user_if_api_key_valid(&data.auth_service, req.api_key.clone()).await?;

    // get connection
    let con: &mut tokio_postgres::Client = &mut *data.pool.get().await.map_err(report_pool_err)?;

    // get user messages
    let sleep_events = sleep_event_service::query(con, req.into_inner())
        .await
        .map_err(report_postgres_err)?;

    // return
    let mut resp_sleep_events = vec![];
    for u in sleep_events.into_iter() {
        resp_sleep_events.push(fill_sleep_event(u));
    }

    Ok(web::Json(resp_sleep_events))
}

// allows you to submit the message in parts
pub async fn ws_submit_user_message(
    data: web::Data<AppData>,
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<request::UserMessageSubmitProps>,
) -> Result<impl Responder, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;
    // spawn websocket handler (and don't await it) so that the response is returned immediately
    rt::spawn(manage_user_message::submit_user_message_ws(
        data, session, msg_stream, query,
    ));
    Ok(res)
}

// allows you to receive the message in parts
pub async fn ws_receive_user_message(
    data: web::Data<AppData>,
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<request::UserMessageReceiveProps>,
) -> Result<impl Responder, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;
    // spawn websocket handler (and don't await it) so that the response is returned immediately
    rt::spawn(manage_user_message::receive_user_message_ws(
        data, session, msg_stream, query,
    ));
    Ok(res)
}

pub async fn get_recent_user_message_id(
    data: web::Data<AppData>,
    query: web::Query<request::GetRecentUserMessageIdProps>,
) -> Result<impl Responder, Error> {
    let con: &mut tokio_postgres::Client = &mut *data.pool.get().await.map_err(report_pool_err)?;
    // get user messages
    let user_message = user_message_service::get_recent_by_target_id(con, query.target_user_id)
        .await
        .map_err(report_postgres_err)?
        .ok_or(AppError::NotFound)?;
    // just return the number
    Ok(user_message.user_message_id.to_string())
}

pub async fn query_params_sleep_event_new(
    data: web::Data<AppData>,
    query: web::Query<request::QueryParamsSleepEventProps>,
) -> Result<impl Responder, Error> {
    let con: &mut tokio_postgres::Client = &mut *data.pool.get().await.map_err(report_pool_err)?;
    // get user messages
    let sleep_event = sleep_event_service::add(con, query.creator_user_id)
        .await
        .map_err(report_postgres_err)?;
    // just return the number
    Ok(web::Json(fill_sleep_event(sleep_event)))
}
