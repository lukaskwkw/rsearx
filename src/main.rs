use std::{
    fs::File,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use log::*;
use simplelog::*;

use handlers::search::search;
use searx_client::SearxClient;
mod searx_client;
mod handlers {
    pub mod search;
}
mod filter;

pub struct Cache {
    creation_time: SystemTime,
    ttl: Duration,
    instances: Vec<String>,
}

pub const HOUR: u32 = 60 * 60;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("rsearx.log").unwrap(),
        ),
    ])
    .unwrap();
    info!("Logger initialized!");
    let base_url = "https://searx.space/".to_string();
    let client = SearxClient::new(base_url);
    let client = Data::new(client);
    let cache = Cache {
        instances: Vec::new(),
        creation_time: SystemTime::now(),
        ttl: Duration::from_secs(HOUR.into()),
    };
    let cache = Data::new(Mutex::new(cache));

    HttpServer::new(move || {
        App::new()
            .route("/search", web::get().to(search))
            .app_data(client.clone())
            .app_data(cache.clone())
    })
    .bind(("127.0.0.1", 8095))?
    .run()
    .await
}
// TODO fetch again after every hour
