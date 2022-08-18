use std::{fs::File, sync::Mutex};

use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use log::*;
use simplelog::*;

use handlers::search::search;
use searx_client::SearxClient;
use serde_json::{Map, Value};
mod searx_client;
mod handlers {
    pub mod search;
}
mod filter;

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
    let instances: Vec<String> = Vec::new();
    let instances = Data::new(Mutex::new(instances));

    HttpServer::new(move || {
        App::new()
            .route("/search", web::get().to(search))
            .app_data(client.clone())
            .app_data(instances.clone())
    })
    .bind(("127.0.0.1", 8095))?
    .run()
    .await
}
// TODO fetch again after every hour
