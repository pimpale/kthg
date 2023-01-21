CREATE DATABASE kthg;
\c kthg;

-- Table Structure
-- Primary Key
-- Creation Time
-- Creator User Id (if applicable)
-- Everything else



drop table if exists user_message cascade;
create table user_message(
  user_message_id bigserial primary key,
  creation_time bigint not null default extract(epoch from now()) * 1000,
  creator_user_id bigint not null,
  target_user_id bigint not null,
  audio_data text not null
);

create view recent_user_message_by_creator_target_id as
  select um.* from user_message um
  inner join (
    select max(user_message_id) id 
    from user_message
    group by creator_user_id, target_user_id
  ) maxids
  on maxids.id = um.user_message_id;


drop table if exists sleep_event cascade;
create table sleep_event(
  sleep_event_id bigserial primary key,
  creation_time bigint not null default extract(epoch from now()) * 1000,
  creator_user_id bigint not null
);









-- drop table if exists checkpoint cascade;
-- create table checkpoint(
--   checkpoint_id bigserial primary key,
--   creation_time bigint not null default extract(epoch from now()) * 1000,
--   creator_user_id bigint not null
-- );
-- 
-- drop table if exists live_task cascade;
-- create table live_task(
--   checkpoint_id bigint not null references checkpoint(checkpoint_id),
--   live_task_id text not null,
--   position i64 not null,
--   value text not null
-- );
-- 
-- drop table if exists finished_task cascade;
-- create table finished_task(
--   checkpoint_id bigint not null references checkpoint(checkpoint_id),
--   finished_task_id text not null,
--   position i64 not null,
--   value text not null,
--   status bigint not null
-- );
