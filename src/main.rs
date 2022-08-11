use actix_web::{get, App, HttpResponse, HttpServer, Responder};

use rand::{self, thread_rng, Rng};
use serde_json::Value;

#[get("/")]
async fn hello() -> impl Responder {
    let body: Value = reqwest::get("https://searx.space/data/instances.json")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // println!("body = {:?}", body["instances"]["https://anon.sx/"]);
    // println!("xd {}", body.as_object().unwrap().len());
    let instances = &body["instances"].as_object().unwrap();
    println!("instanes len {}", instances.len());
    let best_grade_instances: Vec<_> = instances
        .iter()
        .filter_map(|k| {
            let grade: String = k.1["html"]["grade"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let network_type: String = k.1["network_type"].as_str().unwrap_or_default().to_string();
            // println!("grade {}, n_type {}", grade, network_type);
            if ["C", "V"].contains(&&grade[..]) && network_type == "normal" {
                Some(k.0)
            } else {
                None
            }
            // println!("k = {:#?}", k.0);
        })
        .collect();
    println!("best grades len {}", best_grade_instances.len());
    best_grade_instances.iter().for_each(|what| {
        println!("{}", what);
    });

    let mut rng = thread_rng();
    let random_plus: u32 = rng.gen_range(0..best_grade_instances.len() as u32);
    let body = reqwest::get(
        best_grade_instances
            .get(random_plus as usize)
            .unwrap()
            .to_string(),
    )
    .await
    .unwrap()
    .text()
    .await
    .unwrap();
    HttpResponse::Ok().body(body)
    // HttpResponse::Ok().body(body)
}

fn convert_html_urls_to_absolute(body: &String, url: &String) -> String {
    body.replace("href=\"/", &format!("href=\"{}", url))
        .replace("src=\"/{}", &format!("href=\"{}", url))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 8095))?
        .run()
        .await
}
// https://searx.space/data/instances.json
