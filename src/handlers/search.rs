use crate::{searx_client::SearxClient, filter::get_filtered_urls};
use actix_web::{
    web::{self, Data},
    HttpResponse, Responder,
};
use rand::{self, thread_rng, Rng};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Query {
    q: Option<String>,
}

pub async fn search(params: web::Query<Query>, client: Data<SearxClient>) -> impl Responder {
    let body = client.get_instances().await;
    let instances = &body["instances"].as_object().unwrap();
    println!("instanes len {}", instances.len());
    let best_grade_instance_urls = get_filtered_urls(instances);
    println!("best grades len {}", best_grade_instance_urls.len());

    let url = get_random_instance_url(best_grade_instance_urls);
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
