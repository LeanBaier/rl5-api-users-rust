use std::env;

use actix_web::{App, get, HttpResponse, HttpServer, Responder, web};
use actix_web::middleware::Logger;
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use env_logger::Env;
use r2d2::Pool;

mod auth;
mod db;
mod model;
mod schema;
mod users;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let auth_validator_func = move |req, credentials| async {
        let auth_validator = auth::AuthValidator::new("USER".to_string());
        auth_validator.validator(req, credentials)
    };
    let auth = HttpAuthentication::bearer(auth_validator_func);
    let api_port = env::var("API_PORT").expect("API_PORT must be set.").parse().expect("API_PORT must be a number.");
    let api_host = env::var("API_HOST").expect("API_HOST must be set.");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(health)
            .service(
                web::scope("/api/v1/index")
                    .wrap(auth.clone())
                    .service(index),
            )
            .service(
                web::scope("/api/v1")
                    .service(users::register_user)
                    .service(users::refresh_auth)
                    .service(users::login),
            )
    })
    .bind((api_host, api_port))?
    .run()
    .await
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello World with Security!"
}

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().body("")
}