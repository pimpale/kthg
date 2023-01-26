#[derive(Clone, Debug)]
pub struct UserMessage {
    pub user_message_id: i64,
    pub creation_time: i64,
    pub creator_user_id: i64,
    pub target_user_id: i64,
    pub audio_data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SleepEvent {
    pub sleep_event_id: i64,
    pub creation_time: i64,
    pub creator_user_id: i64,
}
