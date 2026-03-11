use actix_web::{middleware, web, App, HttpServer};
use aws_config::BehaviorVersion;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use dotenv::dotenv;
use log::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

#[derive(OpenApi)]
#[openapi(
    info(
        title = "VolunteerMatch Service",
        version = "1.0.0",
        description = "Microservice สำหรับจับคู่อาสาสมัครกับงานในระบบตอบสนองภัยพิบัติ"
    ),
    paths(
        handlers::health::health_check,
        handlers::volunteer::register_volunteer,
        handlers::volunteer::update_location,
        handlers::volunteer::get_location,
        handlers::task::create_task,
        handlers::task::search_tasks,
        handlers::match_handler::match_volunteer,
    ),
    components(schemas(
        models::volunteer::RegisterVolunteerRequest,
        models::volunteer::RegisterVolunteerResponse,
        models::volunteer::UpdateLocationRequest,
        models::volunteer::LocationResponse,
        models::task::CreateTaskRequest,
        models::task::TaskSummary,
        models::match_model::MatchVolunteerRequest,
        models::match_model::MatchResponse,
    )),
    tags(
        (name = "Health", description = "Health check"),
        (name = "Volunteers", description = "ลงทะเบียนและติดตาม GPS อาสาสมัคร"),
        (name = "Tasks", description = "สร้างและค้นหางาน"),
        (name = "Matches", description = "จับคู่อาสากับงาน"),
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    info!("Database connected and migrations applied");

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
    info!("Swagger UI: http://{}:{}/swagger-ui/", host, port);

    let openapi = ApiDoc::openapi();

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
            // Swagger UI
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            // API routes
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
