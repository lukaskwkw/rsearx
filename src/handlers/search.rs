use std::sync::Mutex;

use crate::{filter::get_filtered_urls, searx_client::SearxClient};
use actix_web::{
    web::{self, Data},
    HttpResponse, Responder,
};
use rand::{self, thread_rng, Rng};
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize)]
pub struct Query {
    q: Option<String>,
}

pub async fn search(
    params: web::Query<Query>,
    instances: Data<Mutex<Map<String, Value>>>,
    client: Data<SearxClient>,
) -> impl Responder {
    {
        let is_empty;
        {
            let instances_guard = instances.lock().unwrap();
            is_empty = instances_guard.is_empty();
        }
        // drop(instances_guard); clippy has some issues with drop so using bracket instead { }
        if is_empty {
            let fetched_instances = client.fetch_instances().await;
            let mut instances_guard = instances.lock().unwrap();
            fetched_instances.iter().for_each(|inst| {
                instances_guard.insert(inst.0.to_string(), inst.1.clone());
            })
        };
    }
    let url;
    {
        let instances = instances.lock().unwrap();
        println!("instanes len {}", instances.len());
        let best_grade_instance_urls = get_filtered_urls(&instances);
        println!("best grades len {}", best_grade_instance_urls.len());

        url = get_random_instance_url(best_grade_instance_urls);
    }
    let body = client
        .get_instance_search_body(&url, &params.q.clone().unwrap_or_default())
        .await;
    HttpResponse::Ok().body(body)
}

fn get_random_instance_url(best_grade_instance_urls: Vec<&String>) -> String {
    let mut rng = thread_rng();
    let random_plus: u32 = rng.gen_range(0..best_grade_instance_urls.len() as u32);
    let url = best_grade_instance_urls
        .get(random_plus as usize)
        .unwrap()
        .to_string();
    url
}
