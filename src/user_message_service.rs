use super::db_types::*;
use tokio_postgres::GenericClient;

impl From<tokio_postgres::row::Row> for UserMessage {
    // select * from userMessage order only, otherwise it will fail
    fn from(row: tokio_postgres::Row) -> UserMessage {
        UserMessage {
            user_message_id: row.get("user_message_id"),
            creation_time: row.get("creation_time"),
            creator_user_id: row.get("creator_user_id"),
            target_user_id: row.get("target_user_id"),
            audio_data: row.get("audio_data"),
        }
    }
}

pub async fn add(
    con: &mut impl GenericClient,
    creator_user_id: i64,
    target_user_id: i64,
    audio_data: String,
) -> Result<UserMessage, tokio_postgres::Error> {
    let row = con
        .query_one(
            "INSERT INTO
             user_message(
                 creator_user_id,
                 target_user_id,
                 audio_data
             )
             VALUES($1, $2, $3)
             RETURNING user_message_id, creation_time
            ",
            &[&creator_user_id, &target_user_id, &audio_data],
        )
        .await?;

    // return userMessage
    Ok(UserMessage {
        user_message_id: row.get(0),
        creation_time: row.get(1),
        creator_user_id,
        target_user_id,
        audio_data,
    })
}

pub async fn get_by_user_message_id(
    con: &mut impl GenericClient,
    user_message_id: i64,
) -> Result<Option<UserMessage>, tokio_postgres::Error> {
    let result = con
        .query_opt(
            "SELECT * FROM user_message WHERE user_message_id=$1",
            &[&user_message_id],
        )
        .await?
        .map(|x| x.into());
    Ok(result)
}

pub async fn query(
    con: &mut impl GenericClient,
    props: crate::request::UserMessageViewProps,
) -> Result<Vec<UserMessage>, tokio_postgres::Error> {
    let sql = [
        if props.only_recent {
            "SELECT um.* FROM recent_user_message_by_creator_target_id um"
        } else {
            "SELECT um.* FROM user_message_t um"
        },
        " WHERE 1 = 1",
        " AND ($1::bigint[] IS NULL OR um.user_message_id IN $1)",
        " AND ($2::bigint   IS NULL OR um.creation_time >= $2)",
        " AND ($3::bigint   IS NULL OR um.creation_time <= $3)",
        " AND ($4::bigint[] IS NULL OR um.creator_user_id IN $4)",
        " AND ($5::bigint[] IS NULL OR um.target_user_id IN $5)",
        " ORDER BY um.user_message_id",
    ]
    .join("");

    let stmnt = con.prepare(&sql).await?;

    let results = con
        .query(
            &stmnt,
            &[
                &props.user_message_id,
                &props.min_creation_time,
                &props.max_creation_time,
                &props.creator_user_id,
                &props.target_user_id,
            ],
        )
        .await?
        .into_iter()
        .map(|x| x.into())
        .collect();

    Ok(results)
}
