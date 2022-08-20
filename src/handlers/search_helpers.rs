use rand::{self, thread_rng, Rng};

use crate::filter::get_filtered_urls;

use log::{debug, info};

use crate::searx_client::SearxProvider;

use crate::Cache;

use std::sync::{Mutex, Arc};

use actix_web::web::Data;

pub fn get_random_url_from_cache(cache: &Data<Mutex<Cache>>) -> String {
    let cache_guard = cache.lock().unwrap();
    let instance_urls = &cache_guard.instances;
    get_random_instance_url(instance_urls)
}

pub(crate) async fn populate_cache_if_needed(
    cache: &Data<Mutex<Cache>>,
    client: &Data<Arc<dyn SearxProvider>>,
) {
    let should_fetch;
    {
        let cache_guard = cache.lock().unwrap();
        should_fetch = cache_guard.instances.is_empty() || ttl_exceeded(&cache_guard as &Cache);
    }
    // drop(instances_guard); clippy has some issues with drop so using bracket instead { }
    if should_fetch {
        let fetched_instances = client.fetch_instances().await;
        info!("instanes len {}", fetched_instances.len());
        let best_grade_instance_urls = get_filtered_urls(&fetched_instances);
        info!("best grades len {}", best_grade_instance_urls.len());
        let mut cache_guard = cache.lock().unwrap();
        cache_guard.instances = best_grade_instance_urls
            .iter()
            .map(|url| url.to_string())
            .collect::<Vec<String>>();
    }
}

pub(crate) fn get_random_instance_url(best_grade_instance_urls: &Vec<String>) -> String {
    let mut rng = thread_rng();
    let random_plus: u32 = rng.gen_range(0..best_grade_instance_urls.len() as u32);
    let url = best_grade_instance_urls
        .get(random_plus as usize)
        .unwrap()
        .to_string();
    url
}

pub(crate) fn ttl_exceeded(cache: &Cache) -> bool {
    let elapsed = cache.creation_time.elapsed().as_secs();
    debug!("elapsed {elapsed:?}");
    elapsed > cache.ttl.as_secs()
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::{json, Map, Value};

    use super::*;
    use crate::{searx_client::MockSearxProvider, HOUR};
    use core::time::Duration;
    use mock_instant::{Instant, MockClock};

    fn fetch_instances_return_mock() -> Box<dyn Fn() -> Map<String, Value> + Send> {
        let default = || {
            let mut m = Map::new();
            let json = json!({
            "html": {
                "grade": "C",
            },
            "network_type": "normal"
            });
            m.insert("instance".to_string(), json);
            m
        };
        Box::new(default)
    }

    #[test]
    fn ttl_exceeded_test() {
        let creation_time = Instant::now();
        let cache = Cache {
            creation_time,
            instances: Vec::new(),
            ttl: Duration::from_secs(HOUR.into()),
        };
        assert!(!ttl_exceeded(&cache));

        let creation_time = Instant::now();
        let cache = Cache {
            creation_time,
            instances: Vec::new(),
            ttl: Duration::from_secs(25),
        };
        assert!(!ttl_exceeded(&cache));

        MockClock::advance(Duration::from_secs(5));
        let cache = Cache {
            creation_time,
            instances: Vec::new(),
            ttl: Duration::from_secs(2),
        };
        assert!(ttl_exceeded(&cache));

        MockClock::advance(Duration::from_secs(HOUR.into()));
        let cache = Cache {
            creation_time,
            instances: Vec::new(),
            ttl: Duration::from_secs(HOUR.into()),
        };
        assert!(ttl_exceeded(&cache));
    }

    #[actix_rt::test]
    async fn populate_cache_if_needed_fetch_instances_be_called_test() {
        let mut client_mock = MockSearxProvider::new();
        client_mock
            .expect_fetch_instances()
            .return_once(fetch_instances_return_mock());

        let creation_time = Instant::now();
        MockClock::advance(Duration::from_secs(10));
        MockClock::advance(Duration::from_secs(HOUR.into()));
        let cache = Cache {
            instances: vec!["1".to_string()],
            creation_time,
            ttl: Duration::from_secs(HOUR.into()),
        };
        let cache = Data::new(Mutex::new(cache));
        let client_mock = Arc::new(client_mock) as Arc<dyn SearxProvider>;
        let client_mock = Data::new(client_mock);
        populate_cache_if_needed(&cache, &client_mock).await;
        let cache_guard = cache.lock().unwrap();
        let instances = &cache_guard.instances;

        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0], "instance".to_string());
    }

    #[actix_rt::test]
    async fn populate_cache_if_needed_fetch_instances_be_called_test2() {
        let mut client_mock = MockSearxProvider::new();
        client_mock
            .expect_fetch_instances()
            .return_once(fetch_instances_return_mock());

        let cache = Cache {
            instances: Vec::new(),
            creation_time: Instant::now(),
            ttl: Duration::from_secs(HOUR.into()),
        };
        let cache = Data::new(Mutex::new(cache));
        let client_mock = Arc::new(client_mock) as Arc<dyn SearxProvider>;
        let client_mock = Data::new(client_mock);
        populate_cache_if_needed(&cache, &client_mock).await;
        let instances = &cache.lock().unwrap().instances;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0], "instance".to_string());
    }

    #[actix_rt::test]
    async fn populate_cache_if_needed_fetch_instances_not_be_called_test() {
        let mut client_mock = MockSearxProvider::new();
        client_mock.expect_fetch_instances().never();
        let cache = Cache {
            instances: vec!["1".to_string()],
            creation_time: Instant::now(),
            ttl: Duration::from_secs(HOUR.into()),
        };
        let cache = Data::new(Mutex::new(cache));
        let client_mock = Arc::new(client_mock) as Arc<dyn SearxProvider>;
        let client_mock = Data::new(client_mock);
        populate_cache_if_needed(&cache, &client_mock).await;
        
        let instances = &cache.lock().unwrap().instances;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0], "1".to_string());
    }

    #[actix_rt::test]
    async fn populate_cache_if_needed_fetch_instances_not_be_called_test2() {
        let mut client_mock = MockSearxProvider::new();
        client_mock.expect_fetch_instances().never();
        let creation_time = Instant::now();

        MockClock::advance(Duration::from_secs(1000));
        let cache = Cache {
            instances: vec!["1".to_string()],
            creation_time,
            ttl: Duration::from_secs(HOUR.into()),
        };
        let cache = Data::new(Mutex::new(cache));
        let client_mock = Arc::new(client_mock) as Arc<dyn SearxProvider>;
        let client_mock = Data::new(client_mock);
        populate_cache_if_needed(&cache, &client_mock).await;
        
        let instances = &cache.lock().unwrap().instances;
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0], "1".to_string());
    }
}
