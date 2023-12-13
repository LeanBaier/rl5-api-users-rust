use std::str::FromStr;

use anyhow::anyhow;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Local, NaiveDateTime};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use uuid::Uuid;

use crate::model::{ConnectionModel, RLRole, RLUser};
use crate::schema::rl_users::dsl::rl_users;
use crate::users::{NewUser, UserError};

pub fn save_new_user(conn: &mut PgConnection, new_user: NewUser) -> Result<Uuid, UserError> {
    use crate::schema::rl_users::dsl::*;

    let hashed_password = hash(new_user.password, DEFAULT_COST)
        .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

    let user = rl_users
        .filter(email.eq(new_user.email.clone()))
        .limit(1)
        .load::<RLUser>(conn)
        .map_err(|e| anyhow!("{}", e))?;

    if !user.is_empty() {
        return Err(UserError::EmailNotAvailable);
    }

    let new_user = RLUser {
        id_user: Uuid::new_v4(),
        email: new_user.email.clone(),
        nickname: new_user.nickname.clone(),
        password: hashed_password,
        id_role: 1,
    };

    let new_user_id = new_user.id_user;
    diesel::insert_into(rl_users)
        .values(new_user)
        .execute(conn)
        .map_err(|e| anyhow!("{}", e))?;

    Ok(new_user_id)
}
pub fn generate_new_connection(conn: &mut PgConnection, id: Uuid) -> Result<Uuid, UserError> {
    use crate::schema::connections::dsl::*;
    use crate::schema::rl_users::dsl::id_user as user_id_user;

    let user = rl_users
        .filter(user_id_user.eq(id))
        .select(RLUser::as_select())
        .get_result(conn)
        .map_err(|_| UserError::UserNotFound)?;

    let timestamp = Local::now();
    let new_connection = ConnectionModel {
        id_connection: Uuid::new_v4(),
        id_user: user.id_user,
        connect_at: Some(NaiveDateTime::from_timestamp_opt(timestamp.timestamp(), 0).unwrap()),
        ended_at: None,
    };

    let connection_id = new_connection.id_connection;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let active_connections = connections
            .filter(id_user.eq(id).and(ended_at.is_null()))
            .load::<ConnectionModel>(conn)?;
        for con in active_connections {
            diesel::update(connections.filter(id_connection.eq(con.id_connection)))
                .set(ended_at.eq(Some(
                    NaiveDateTime::from_timestamp_opt(timestamp.timestamp(), 0).unwrap(),
                )))
                .execute(conn)?;
        }
        diesel::insert_into(connections)
            .values(new_connection)
            .execute(conn)?;
        Ok(())
    })
    .map_err(|e| anyhow!("{}", e))?;

    Ok(connection_id)
}

pub fn login(
    conn: &mut PgConnection,
    email_login: String,
    password_login: String,
) -> Result<(RLUser, RLRole), UserError> {
    use crate::schema::rl_role::dsl::id_role as role_id_role;
    use crate::schema::rl_role::dsl::rl_role;
    use crate::schema::rl_users::dsl::*;
    let hashed_password = hash(password_login, DEFAULT_COST)
        .map_err(|e| anyhow!("Failed to hash password: {}", e))?;
    let user = rl_users
        .filter(email.eq(email_login))
        .select(RLUser::as_select())
        .first(conn)
        .map_err(|e| anyhow!("{}", e))?;

    verify(user.password.clone(), &hashed_password).map_err(|_| UserError::InvalidCredentials)?;

    let role = rl_role
        .filter(role_id_role.eq(user.id_role))
        .select(RLRole::as_select())
        .first(conn)
        .map_err(|e| anyhow!("{}", e))?;

    Ok((user, role))
}

pub fn get_role_by_user_id(conn: &mut PgConnection, user_id: String) -> Result<RLRole, UserError> {
    use crate::schema::rl_role::dsl::id_role;
    use crate::schema::rl_role::dsl::rl_role;
    use crate::schema::rl_users::dsl::id_user;

    let user = rl_users
        .filter(id_user.eq(Uuid::from_str(&user_id).map_err(|e| anyhow!("{}", e))?))
        .select(RLUser::as_select())
        .first(conn)
        .map_err(|e| anyhow!("{}", e))?;
    let role = rl_role
        .filter(id_role.eq(user.id_role))
        .select(RLRole::as_select())
        .first(conn)
        .map_err(|e| anyhow!("{}", e))?;

    Ok(role)
}

pub fn validate_connection(
    conn: &mut PgConnection,
    user_id: Uuid,
    connection_id: Uuid,
) -> Result<(), UserError> {
    use crate::schema::connections::dsl::*;
    let connection = connections
        .filter(id_user.eq(user_id).and(id_connection.eq(connection_id)))
        .select(ConnectionModel::as_select())
        .first(conn)
        .map_err(|e| {
            log::info!("{}", e);
            anyhow!("{}", e)
        })?;

    connection
        .ended_at
        .map(|_| Err(UserError::ExpiredToken))
        .unwrap_or(Ok(()))?;

    Ok(())
}
