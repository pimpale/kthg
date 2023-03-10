use std::net::Ipv4Addr;
use std::str::FromStr;

use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;

use auth_service_api::client::AuthService;

mod db_types;
mod handlers;
mod request;
mod response;
mod utils;

mod manage_user_message;

mod sleep_event_service;
mod user_message_service;

static SERVICE: &'static str = "kthg";
static VERSION_MAJOR: i64 = 0;
static VERSION_MINOR: i64 = 0;
static VERSION_REV: i64 = 1;

#[derive(Parser, Debug, Clone)]
#[clap(about, version, author)]
struct Opts {
    #[clap(long)]
    port: u16,
    #[clap(long)]
    database_url: String,
    #[clap(long)]
    auth_service_url: String,
    #[clap(long)]
    app_pub_origin: String,
}

#[derive(Clone)]
pub struct AppData {
    pub auth_service: AuthService,
    pub app_pub_origin: String,
    pub pool: deadpool_postgres::Pool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    env_logger::init();

    let Opts {
        auth_service_url,
        app_pub_origin,
        port,
        database_url,
    } = Opts::parse();

    // connect to postgres
    let postgres_config = tokio_postgres::Config::from_str(&database_url).map_err(|e| {
        log::error!("couldn't parse database_url: {}", e);
        e
    })?;
    log::info!("parsed database url");

    let mgr = deadpool_postgres::Manager::from_config(
        postgres_config,
        tokio_postgres::NoTls,
        deadpool_postgres::ManagerConfig {
            recycling_method: deadpool_postgres::RecyclingMethod::Fast,
        },
    );

    let pool = deadpool_postgres::Pool::builder(mgr)
        .max_size(16)
        .build()
        .map_err(|e| {
            log::error!("couldn't build database connection pool: {}", e);
            e
        })?;

    log::info!("built database connection pool");

    // open connection to auth service
    let auth_service = AuthService::new(&auth_service_url);
    log::info!("connected to auth service");

    // start server
    let data = AppData {
        auth_service,
        app_pub_origin,
        pool,
    };

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // add data
            .app_data(actix_web::web::Data::new(data.clone()))
            // handle info query
            .service(web::resource("/public/info").route(web::route().to(handlers::info)))
            // submit user message
            .service(
                web::resource("/public/user_message/new")
                    .route(web::route().to(handlers::user_message_new)),
            )
            // submit sleep event
            .service(
                web::resource("/public/sleep_event/new")
                    .route(web::route().to(handlers::sleep_event_new)),
            )
            // view user message
            .service(
                web::resource("/public/user_message/view")
                    .route(web::route().to(handlers::user_message_view)),
            )
            // view sleep event
            .service(
                web::resource("/public/sleep_event/view")
                    .route(web::route().to(handlers::sleep_event_view)),
            )
            // get recent id
            .service(
                web::resource("/public/get_recent_user_message_id")
                    .route(web::route().to(handlers::get_recent_user_message_id)),
            )
            // new sleep event via query params
            .service(
                web::resource("/public/query_params_sleep_event_new")
                    .route(web::route().to(handlers::query_params_sleep_event_new)),
            )
            // websocket submit recording
            .service(
                web::resource("/public/ws/submit_user_message")
                    .route(web::route().to(handlers::ws_submit_user_message)),
            )
            // websocket receieve recording
            .service(
                web::resource("/public/ws/receive_user_message")
                    .route(web::route().to(handlers::ws_receive_user_message)),
            )
    })
    .bind((Ipv4Addr::new(0, 0, 0, 0), port))?
    .run()
    .await?;

    Ok(())
}
