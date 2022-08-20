use std::sync::{Mutex, Arc};

use crate::{searx_client::SearxProvider, Cache};
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
    client: Data<Arc<dyn SearxProvider>>,
) -> impl Responder {
    search_helpers::populate_cache_if_needed(&cache, &client).await;
    let url = search_helpers::get_random_url_from_cache(&cache);
    let body = client
        .get_instance_search_body(&url, &params.q.clone().unwrap_or_default())
        .await;
    HttpResponse::Ok().body(body)
}
