use actix_web::{
    web::{self, Data}, HttpResponse, Responder,
};
use rand::{self, thread_rng, Rng};
use serde::Deserialize;
use crate::searx_client::SearxClient;

#[derive(Deserialize)]
pub struct Query {
    q: Option<String>,
}

pub async fn search(_info: web::Query<Query>, client: Data<SearxClient>) -> impl Responder {
    // if let Some(username) = info.username.as_ref() {
    //     let query = format!("Welcome {}!", username);
    //
    //     println!("query {}", query);
    // }
    let body = client.get_instances().await;
    let instances = &body["instances"].as_object().unwrap();
    println!("instanes len {}", instances.len());
    let best_grade_instance_urls: Vec<_> = instances
        .iter()
        .filter_map(|k| {
            let grade: String = k.1["html"]["grade"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let network_type: String = k.1["network_type"].as_str().unwrap_or_default().to_string();
            if ["C", "V"].contains(&&grade[..]) && network_type == "normal" {
                Some(k.0)
            } else {
                None
            }
        })
        .collect();
    println!("best grades len {}", best_grade_instance_urls.len());
    best_grade_instance_urls.iter().for_each(|what| {
        println!("{}", what);
    });

    let mut rng = thread_rng();
    let random_plus: u32 = rng.gen_range(0..best_grade_instance_urls.len() as u32);
    let url = best_grade_instance_urls
        .get(random_plus as usize)
        .unwrap()
        .to_string();
    let body = reqwest::get(&url).await.unwrap().text().await.unwrap();
    let body = convert_html_urls_to_absolute(body, &url);
    HttpResponse::Ok().body(body)
    // HttpResponse::Ok().body(body)
}

fn convert_html_urls_to_absolute(body: String, url: &str) -> String {
    body.replace("href=\"/", &format!("href=\"{}", url))
        .replace("src=\"/", &format!("src=\"{}", url))
        .replace("\"/search", &format!("\"{}search", url))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn convert_html_urls_to_absolute_test() {
        let body =
            r#"<a href="/lola" /> <img src="/heheszki.jpg" /> <form action="/search"></form> "#
                .to_string();
        let after = convert_html_urls_to_absolute(body, "https://test.com/");
        assert_eq!(
            after,
            r#"<a href="https://test.com/lola" /> <img src="https://test.com/heheszki.jpg" /> <form action="https://test.com/search"></form> "#
        );
    }
}
