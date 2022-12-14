use std::sync::{Arc, Mutex};

use crate::{searx_client::SearxProvider, AppConfig, Cache};
use actix_web::{
    web::{self, Data},
    HttpResponse, Responder,
};

use serde::Deserialize;

use super::search_helpers;

#[derive(Deserialize)]
pub struct Query {
    q: Option<String>,
}

pub async fn search(
    params: web::Query<Query>,
    cache: Data<Mutex<Cache>>,
    app_config: Data<Mutex<AppConfig>>,
    client: Data<Arc<dyn SearxProvider>>,
) -> impl Responder {
    match search_helpers::populate_cache_if_needed(&cache, &client, &app_config).await {
        Ok(it) => it,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };
    let url = search_helpers::get_random_url_from_cache(&cache);
    let body = match client
        .get_instance_search_body(&url, &params.q.clone().unwrap_or_default())
        .await
    {
        Ok(it) => it,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };
    HttpResponse::Ok().body(body)
}
