use reqwest::Client;
use serde_json::Value;

#[derive(Debug)]
pub struct SearxClient {
    http_client: Client,
    base_url: String,
}

impl SearxClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http_client: Client::new(),
        }
    }
    pub async fn get_instances(&self) -> Value {
        self.http_client
            .get("https://searx.space/data/instances.json")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}
