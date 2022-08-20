#[cfg(test)]
use mock_instant::Instant;
use std::{fs::File, sync::{Mutex, Arc}, time::Duration};

use actix_web::{
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
mod searx_client;
mod handlers {
    pub mod search;
    pub mod search_helpers;
}
mod filter;

#[derive(Debug)]
pub struct Cache {
    creation_time: Instant,
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
    let client: Data<Arc<dyn SearxProvider>> = Data::new(Arc::new(client));
    let cache = Cache {
        instances: Vec::new(),
        creation_time: Instant::now(),
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
