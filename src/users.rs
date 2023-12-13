use std::str::FromStr;

use actix_web::body::BoxBody;
use actix_web::error::BlockingError;
use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, ResponseError};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth, db, DbPool};

pub type Result<T> = std::result::Result<T, UserError>;
#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("An unspecified internal error ocurred: {0}")]
    InternalError(#[from] anyhow::Error),
    #[error("The Email is not available")]
    EmailNotAvailable,
    #[error("User not found")]
    UserNotFound,
    #[error("Expired token.")]
    ExpiredToken,
    #[error("An unspecified internal error ocurred")]
    DatabaseError(#[from] BlockingError),
    #[error("Invalid Credentials")]
    InvalidCredentials,
    #[error("Forbidden")]
    Forbidden
}

impl UserError {
    fn get_error_code(&self) -> String {
        match self {
            UserError::InternalError(_) => "IE-00500".to_string(),
            UserError::EmailNotAvailable => "ENA-00400".to_string(),
            UserError::UserNotFound => "UNF-00404".to_string(),
            UserError::ExpiredToken => "ET-00403".to_string(),
            UserError::DatabaseError(_) => "DE-00500".to_string(),
            UserError::InvalidCredentials => "IC-00400".to_string(),
            UserError::Forbidden => "FB-00401".to_string()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserErrorResponse {
    pub message: String,
    pub status: u16,
    pub timestamp: NaiveDateTime,
    pub internal_code: String,
}

impl From<&UserError> for UserErrorResponse {
    fn from(value: &UserError) -> Self {
        Self {
            message: value.to_string(),
            status: value.status_code().as_u16(),
            timestamp: NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                .unwrap_or_default(),
            internal_code: value.get_error_code(),
        }
    }
}
impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match &self {
            UserError::EmailNotAvailable => StatusCode::BAD_REQUEST,
            UserError::UserNotFound => StatusCode::FORBIDDEN,
            UserError::ExpiredToken => StatusCode::FORBIDDEN,
            UserError::Forbidden => StatusCode::FORBIDDEN,
            UserError::InvalidCredentials => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(UserErrorResponse::from(self))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub nickname: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}
#[post("/users/register")]
pub async fn register_user(
    pool: web::Data<DbPool>,
    new_user: web::Json<NewUser>,
) -> Result<HttpResponse> {
    let (new_user_id, new_connection_id) = web::block(move || {
        let mut conn = pool.get().expect("Couldn't get db connection from pool.");
        let new_user_id = db::save_new_user(&mut conn, new_user.into_inner());
        match new_user_id {
            Err(e) => (Err(e), None),
            Ok(user_id) => {
                let new_connection_id = db::generate_new_connection(&mut conn, user_id);
                (new_user_id, Some(new_connection_id))
            }
        }
    })
    .await?;

    let new_user_id = new_user_id?;
    let new_connection_id = new_connection_id.unwrap()?;

    let (access_token, refresh_token, expire_in) =
        auth::generate_tokens(new_user_id, new_connection_id, vec!["USER".to_string()])?;
    Ok(HttpResponse::Ok().json(TokenResponse {
        expires_in: expire_in,
        access_token,
        refresh_token,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[post("/users/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    login_request: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let mut conn = pool
        .get()
        .map_err(|_| anyhow!("Couldn't get db connection from pool."))?;
    let email = login_request.email.clone();
    let password = login_request.password.clone();
    let (user, role) = web::block(move || db::login(&mut conn, email, password)).await??;

    let mut conn = pool
        .get()
        .map_err(|_| anyhow!("Couldn't get db connection from pool."))?;

    let user_id = user.id_user;
    let connection = web::block(move || db::generate_new_connection(&mut conn, user_id)).await??;

    let (access_token, refresh_token, expire_in) =
        auth::generate_tokens(user_id, connection, vec![role.description])?;
    Ok(HttpResponse::Ok().json(TokenResponse {
        expires_in: expire_in,
        access_token,
        refresh_token,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshAuthRequest {
    pub refresh_token: String,
}
#[post("/users/token")]
pub async fn refresh_auth(
    pool: web::Data<DbPool>,
    refresh_auth_request: web::Json<RefreshAuthRequest>,
) -> Result<HttpResponse> {
    let claims = auth::get_claims_and_validate(refresh_auth_request.refresh_token.clone())?;
    let user_id = Uuid::from_str(&claims.user_id).map_err(|e| anyhow!("{}", e))?;
    let connection_id = Uuid::from_str(&claims.connection_id).map_err(|e| anyhow!("{}", e))?;

    let mut conn = pool
        .get()
        .map_err(|_| anyhow!("Couldn't get db connection from pool."))?;

    web::block(move || db::validate_connection(&mut conn, user_id, connection_id)).await??;

    let mut conn = pool
        .get()
        .map_err(|_| anyhow!("Couldn't get db connection from pool."))?;

    let role = web::block(move || db::get_role_by_user_id(&mut conn, claims.user_id.clone()))
        .await??;

    let mut conn = pool
        .get()
        .map_err(|_| anyhow!("Couldn't get db connection from pool."))?;

    let connection = web::block(move || db::generate_new_connection(&mut conn, user_id))
        .await??;

    let (access_token, refresh_token, expire_in) =
        auth::generate_tokens(user_id, connection, vec![role.description])?;
    Ok(HttpResponse::Ok().json(TokenResponse {
        expires_in: expire_in,
        access_token,
        refresh_token,
    }))
}
