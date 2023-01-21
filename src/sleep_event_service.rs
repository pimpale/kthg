use super::db_types::*;
use todoproxy_api::StateSnapshot;
use tokio_postgres::GenericClient;

impl From<tokio_postgres::row::Row> for SleepEvent {
    // select * from sleepEvent order only, otherwise it will fail
    fn from(row: tokio_postgres::Row) -> SleepEvent {
        SleepEvent {
            sleep_event_id: row.get("sleep_event_id"),
            creation_time: row.get("creation_time"),
            creator_user_id: row.get("creator_user_id"),
        }
    }
}

pub async fn add(
    con: &mut impl GenericClient,
    creator_user_id: i64,
) -> Result<SleepEvent, tokio_postgres::Error> {
    let row = con
        .query_one(
            "INSERT INTO
             sleep_event(
                 creator_user_id,
             )
             VALUES($1)
             RETURNING sleep_event_id, creation_time
            ",
            &[&creator_user_id],
        )
        .await?;

    // return sleepEvent
    Ok(SleepEvent {
        sleep_event_id: row.get(0),
        creation_time: row.get(1),
        creator_user_id,
    })
}

pub async fn get_by_sleep_event_id(
    con: &mut impl GenericClient,
    sleep_event_id: i64,
) -> Result<Option<SleepEvent>, tokio_postgres::Error> {
    let result = con
        .query_opt(
            "SELECT * FROM sleep_event WHERE sleep_event_id=$1",
            &[&sleep_event_id],
        )
        .await?
        .map(|x| x.into());
    Ok(result)
}

pub async fn get_recent_by_user_id(
    con: &mut impl GenericClient,
    creator_user_id: i64,
) -> Result<Option<SleepEvent>, tokio_postgres::Error> {
    let result = con
        .query_opt(
            "SELECT * FROM recent_sleep_event_by_user_id WHERE creator_user_id=$1",
            &[&creator_user_id],
        )
        .await?
        .map(|x| x.into());
    Ok(result)
}
