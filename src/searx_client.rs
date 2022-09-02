#[cfg(test)]
use mockall::automock;

use anyhow::anyhow;
use async_trait::async_trait;
use log::info;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Url,
};
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct SearxClient {
    http_client: Client,
    base_url: Url,
}

static AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:103.0) Gecko/20100101 Firefox/103.0";

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SearxProvider: Sync + Send {
    async fn fetch_instances(&self) -> anyhow::Result<Map<String, Value>>;
    async fn get_instance_search_body(
        &self,
        instance_url: &str,
        query: &str,
    ) -> anyhow::Result<String>;
}

#[async_trait]
impl SearxProvider for SearxClient {
    async fn fetch_instances(&self) -> anyhow::Result<Map<String, Value>> {
        info!("start fetching");
        let url = Url::join(&self.base_url, "data/instances.json")?;
        let body: Value = self.http_client.get(url).send().await?.json().await?;
        info!("end of fetching");
        let instances: Map<String, Value> = body["instances"]
            .as_object()
            .ok_or_else(|| anyhow!("no instances prop"))?
            .clone();
        Ok(instances)
    }
    async fn get_instance_search_body(
        &self,
        instance_url: &str,
        query: &str,
    ) -> anyhow::Result<String> {
        let url = get_insance_search_url(instance_url, query);
        let mut headers = HeaderMap::new();
        url.host_str()
            .map(|url| {
                headers.insert(header::HOST, HeaderValue::from_str(url).unwrap());
            })
            .unwrap_or_default();
        headers.insert(header::USER_AGENT, HeaderValue::from_str(AGENT).unwrap());
        headers.insert(header::TRANSFER_ENCODING, header::TRAILER.into());

        headers.insert(header::ACCEPT, HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8").unwrap());
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_str("pl,en-US;q=0.7,en;q=0.3").unwrap(),
        );
        // headers.insert(
        //         header::ACCEPT_LANGUAGE,
        //         HeaderValue::from_str("en-US,en;q=0.5").unwrap(),
        //     );
        let body = self
            .http_client
            .get(url)
            .headers(headers)
            // .header("Connection", "keep")
            .send()
            .await?
            .text()
            .await?;
        Ok(convert_html_urls_to_absolute(body, instance_url))
    }
}

impl SearxClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: Url::parse(&base_url).unwrap(),
            http_client: Client::new(),
        }
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
    info!("instance full url {url}");

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
