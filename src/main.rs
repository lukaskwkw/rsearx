use filter::Filter;
#[cfg(test)]
use mock_instant::Instant;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_files as afs;
use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use log::*;
use simplelog::*;
#[cfg(not(test))]
use std::time::Instant;

use handlers::search::search;
use searx_client::SearxClient;

use crate::searx_client::SearxProvider;
use handlers::save::save;
mod filter;
mod handlers;
mod searx_client;

#[derive(Debug)]
pub struct Cache {
    creation_time: Instant,
    ttl: Duration,
    instances: Vec<String>,
}

pub const HOUR: u32 = 60 * 60;

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct AppConfig {
    server_conf: Option<String>,
    filter: Option<Filter>,
}

pub static CONFIG_FILENAME: &str = "config.json";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_config = fs::read_to_string(CONFIG_FILENAME)
        .map(|content| serde_json::from_str::<AppConfig>(&content).unwrap_or_default())
        .unwrap_or_default();
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
    info!("app_conf: {app_config:?}");
    let base_url = "https://searx.space/".to_string();
    let client = SearxClient::new(base_url);
    let client: Data<Arc<dyn SearxProvider>> = Data::new(Arc::new(client));
    let cache = Cache {
        instances: Vec::new(),
        creation_time: Instant::now(),
        ttl: Duration::from_secs(HOUR.into()),
    };
    let cache = Data::new(Mutex::new(cache));
    let app_config = Data::new(Mutex::new(app_config));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/search", web::get().to(search))
            .route("/save", web::post().to(save))
            .service(afs::Files::new("/", "./dist").index_file("index.html")) // this has to be
            // after all other
            // routes
            .app_data(client.clone())
            .app_data(cache.clone())
            .app_data(app_config.clone())
    })
    .bind(("127.0.0.1", 8095))?
    .run()
    .await
}
