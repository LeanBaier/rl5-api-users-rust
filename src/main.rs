use std::env;

use actix_web::{App, get, HttpServer, Responder, web};
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

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
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
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello World with Security!"
}
