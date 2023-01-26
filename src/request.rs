use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessageNewProps {
    pub target_user_id: i64,
    pub audio_data: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepEventNewProps {
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessageViewProps {
    pub user_message_id: Option<Vec<i64>>,
    pub min_creation_time: Option<i64>,
    pub max_creation_time: Option<i64>,
    pub creator_user_id: Option<Vec<i64>>,
    pub target_user_id: Option<Vec<i64>>,
    pub only_recent: bool,
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepEventViewProps {
    pub sleep_event_id: Option<Vec<i64>>,
    pub min_creation_time: Option<i64>,
    pub max_creation_time: Option<i64>,
    pub creator_user_id: Option<Vec<i64>>,
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessageSubmitProps {
    pub target_user_id: i64,
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessageReceiveProps {
    pub user_message_id: i64,
    pub api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRecentUserMessageIdProps {
    pub target_user_id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParamsSleepEventProps {
    pub creator_user_id: i64,
}
