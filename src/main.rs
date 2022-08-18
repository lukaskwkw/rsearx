use actix_web::{
    web::{self, Data},
    App, HttpServer,
};

use handlers::search::search;
use searx_client::SearxClient;
mod searx_client;
mod handlers {
    pub mod search;
}
mod filter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let base_url = "https://searx.space/".to_string();
    let client = SearxClient::new(base_url);
    let client = Data::new(client);
    HttpServer::new(move || {
        App::new()
            .route("/search", web::get().to(search))
            .app_data(client.clone())
    })
    .bind(("127.0.0.1", 8095))?
    .run()
    .await
}
