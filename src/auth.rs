use std::env;

use actix_web::dev::ServiceRequest;
use actix_web::Error;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use anyhow::anyhow;
use chrono::{NaiveDateTime, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::users::UserError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenClaims {
    pub exp: i64,
    pub iss: String,
    pub sub: String,
    pub user_id: String,
    pub connection_id: String,
    pub roles: Vec<String>,
}

pub fn generate_tokens(
    user_id: Uuid,
    connection_id: Uuid,
    roles: Vec<String>,
) -> anyhow::Result<(String, String, i64)> {
    let header = Header::new(Algorithm::HS256);
    let access_duration = env::var("ACCESS_TOKEN_EXP_SEC")
        .expect("ACCESS_TOKEN_EXP_SEC must be set.")
        .parse()
        .expect("ACCESS_TOKEN_EXP_SEC must be a number.");
    let duration = chrono::Duration::seconds(access_duration);
    let expiration_access = chrono::Utc::now() + duration;
    let claims = TokenClaims {
        sub: "RLClient".to_string(),
        iss: "RLBackend".to_string(),
        exp: expiration_access.timestamp(),
        user_id: user_id.clone().to_string(),
        connection_id: connection_id.clone().to_string(),
        roles: roles.clone(),
    };
    let access_token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(get_secret().as_ref()),
    )
    .map_err(|e| anyhow!("{}", e))?;
    let refresh_duration = env::var("REFRESH_TOKEN_EXP_DAY")
        .expect("REFRESH_TOKEN_EXP_DAY must be set.")
        .parse()
        .expect("REFRESH_TOKEN_EXP_DAY must be a number.");
    let duration = chrono::Duration::days(refresh_duration);
    let expiration = chrono::Utc::now() + duration;
    let claims = TokenClaims {
        sub: "RLClient".to_string(),
        iss: "RLBackend".to_string(),
        exp: expiration.timestamp(),
        user_id: user_id.to_string(),
        connection_id: connection_id.to_string(),
        roles,
    };
    let refresh_token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(get_secret().as_ref()),
    )
    .map_err(|e| anyhow!("{}", e))?;

    let expire_in = expiration_access.timestamp() - chrono::Utc::now().timestamp();

    Ok((access_token, refresh_token, expire_in))
}

pub fn get_claims_and_validate(token: String) -> anyhow::Result<TokenClaims> {
    let secret_key = get_secret();
    let token = token.replace("Bearer ", "");
    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    )?;
    Ok(claims.claims)
}

#[derive(Debug, Clone)]
pub struct AuthValidator {
    pub valid_role: String,
}

impl AuthValidator {
    pub fn new(role: String) -> Self {
        Self { valid_role: role }
    }
    pub fn validator(
        &self,
        req: ServiceRequest,
        credentials: BearerAuth,
    ) -> Result<ServiceRequest, (Error, ServiceRequest)> {
        let secret_key = get_secret();
        let token = credentials.token().replace("Bearer ", "");
        let claims = decode::<TokenClaims>(
            &token,
            &DecodingKey::from_secret(secret_key.as_ref()),
            &Validation::default(),
        );

        match claims {
            Ok(val) => {
                if !val
                    .claims
                    .roles
                    .iter()
                    .any(|r| r.eq_ignore_ascii_case(&self.valid_role))
                {
                    return Err((Error::from(UserError::Forbidden), req));
                }

                Ok(req)
            }
            Err(_) => Err((Error::from(UserError::Forbidden), req)),
        }
    }
}

fn get_secret() -> String {
    env::var("SECRET").expect("SECRET must be set")
}
