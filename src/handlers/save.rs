use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};

use crate::{
    filter::{get_filtered_urls, Filter, Timings},
    searx_client::SearxProvider,
    AppConfig, Cache, CONFIG_FILENAME,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use anyhow::{self, Result};
// use actix_web::Result;

use log::{info};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SaveDto {
    search: Option<String>,
    google: Option<String>,
    wikipedia: Option<String>,
    initial: Option<String>,
    grades: Option<Vec<String>>,
    min_version: Option<String>,
    max_version: Option<String>,
}

pub async fn save(
    body: Json<SaveDto>,
    cache: Data<Mutex<Cache>>,
    client: Data<Arc<dyn SearxProvider>>,
    app_config: Data<Mutex<AppConfig>>,
) -> impl Responder {
    let fetched_instances = match client.fetch_instances().await {
        Ok(it) => it,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };
    info!("instanes len {}", fetched_instances.len());
    let filter = Filter {
        response_times: Some(Timings {
            search: body.search.as_ref().and_then(|text| text.parse().ok()),
            google: body.google.as_ref().and_then(|text| text.parse().ok()),
            wikipedia: body.wikipedia.as_ref().and_then(|text| text.parse().ok()),
            initial: body.initial.as_ref().and_then(|text| text.parse().ok()),
        }),
        grades: body.grades.clone(),
        ..Filter::default()
    };
    let best_grade_instance_urls = get_filtered_urls(&fetched_instances, &filter);
    info!("best grades len {}", best_grade_instance_urls.len());
    let mut cache_guard = cache.lock().unwrap();
    cache_guard.instances = best_grade_instance_urls
        .iter()
        .map(|url| url.to_string())
        .collect();
    drop(cache_guard);
    let mut app_conf_guard = app_config.lock().unwrap();
    app_conf_guard.filter = Some(filter);
    let app_conf = app_conf_guard.clone();
    drop(app_conf_guard);
    info!("app_conf {app_conf:?}");

    let save_to_file = || -> Result<_, anyhow::Error> {
        let app_conf_str = serde_json::to_string_pretty(&app_conf)?;

        File::create(CONFIG_FILENAME)
            .and_then(|mut file| file.write(app_conf_str.as_bytes()))
            .map(|_| ())?;
        Ok(())
    };
    match save_to_file() {
        Ok(_) => HttpResponse::Ok().body("Data has been saved"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
