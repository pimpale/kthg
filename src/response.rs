use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessage {
    pub creation_time: i64,
    pub creator_user_id: i64,
    pub target_user_id: i64,
    pub audio_data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepEvent {
    pub creation_time: i64,
    pub creator_user_id: i64,
}

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
