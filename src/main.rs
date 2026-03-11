use actix_web::{middleware, web, App, HttpServer};
use aws_config::BehaviorVersion;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use dotenv::dotenv;
use log::info;

mod db;
mod errors;
mod handlers;
mod messaging;
mod models;

use db::Db;
use messaging::Messenger;

pub struct AppState {
    pub db: Arc<Db>,
    pub messenger: Arc<Messenger>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // PostgreSQL connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations on startup
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    info!("Database connected and migrations applied");

    // AWS config — uses EC2 IAM role automatically on AWS
    let aws_cfg = aws_config::defaults(BehaviorVersion::latest())
        .load()
        .await;

    let sns_client = aws_sdk_sns::Client::new(&aws_cfg);

    let db = Arc::new(Db::new(pool));
    let messenger = Arc::new(Messenger::new(sns_client));

    let host = std::env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap_or(8080);

    info!("VolunteerMatch Service starting on {}:{}", host, port);

    HttpServer::new(move || {
        let state = web::Data::new(AppState {
            db: Arc::clone(&db),
            messenger: Arc::clone(&messenger),
        });

        App::new()
            .app_data(state)
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _req| {
                        let msg = format!("invalid JSON: {}", err);
                        actix_web::error::InternalError::from_response(
                            err,
                            actix_web::HttpResponse::BadRequest()
                                .json(serde_json::json!({ "error": msg })),
                        )
                        .into()
                    }),
            )
            .wrap(middleware::Logger::default())
            .service(handlers::health::health_check)
            .service(handlers::volunteer::register_volunteer)
            .service(handlers::volunteer::update_location)
            .service(handlers::volunteer::get_location)
            .service(handlers::task::create_task)
            .service(handlers::task::search_tasks)
            .service(handlers::match_handler::match_volunteer)
    })
    .bind((host, port))?
    .run()
    .await
}
