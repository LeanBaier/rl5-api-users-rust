use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::*;

#[derive(Debug, PartialEq, Queryable, Selectable, Insertable)]
#[diesel(table_name = rl_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RLUser {
    pub id_user: Uuid,
    pub email: String,
    pub nickname: String,
    pub password: String,
    pub id_role: i32,
}

#[derive(Debug, PartialEq, Queryable, Selectable)]
#[diesel(table_name = rl_role)]
pub struct RLRole {
    pub id_role: i32,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Insertable)]
#[diesel(table_name = connections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConnectionModel {
    pub id_connection: Uuid,
    pub id_user: Uuid,
    pub connect_at: Option<NaiveDateTime>,
    pub ended_at: Option<NaiveDateTime>,
}
