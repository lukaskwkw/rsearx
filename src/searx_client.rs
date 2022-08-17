use reqwest::{Client, Url};
use serde_json::Value;

#[derive(Debug)]
pub struct SearxClient {
    http_client: Client,
    base_url: Url,
}

impl SearxClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: Url::parse(&base_url).unwrap(),
            http_client: Client::new(),
        }
    }
    pub async fn get_instances(&self) -> Value {
        let url = Url::join(&self.base_url, "data/instances.json").unwrap();
        self.http_client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
    pub async fn get_instance_search_body(&self, instance_url: &str, query: &str) -> String {
        let url = get_insance_search_url(instance_url, query);
        let body = self
            .http_client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        convert_html_urls_to_absolute(body, instance_url)
    }
}

fn convert_html_urls_to_absolute(body: String, url: &str) -> String {
    body.replace("href=\"/", &format!("href=\"{}", url))
        .replace("src=\"/", &format!("src=\"{}", url))
        .replace("\"/search", &format!("\"{}search", url))
}

fn get_insance_search_url(instance_url: &str, query: &str) -> Url {
    let search_route = format!("/search?q={}", query);
    let url = Url::parse(instance_url)
        .unwrap()
        .join(&search_route)
        .unwrap();
    println!("instance full url {url}");

    url
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
    #[test]
    fn get_insance_search_url_test() {
        let instance = "http://searx.jp/";
        let query = "semaphore";
        let expected_url = "http://searx.jp/search?q=semaphore";
        let result = get_insance_search_url(instance, query).to_string();
        assert_eq!(result, expected_url);
    }
}
