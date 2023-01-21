use derive_more::Display;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessage {
    creator_user_id: i64,
    target_user_id: i64,
    creation_time: i64,
    audio_data: String,
}

impl std::error::Error for AppError {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub service: String,
    pub version_major: i64,
    pub version_minor: i64,
    pub version_rev: i64,
    pub app_pub_origin: String,
    pub auth_pub_api_href: String,
    pub auth_authenticator_href: String,
}