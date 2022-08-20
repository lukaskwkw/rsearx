
use rand::{self, thread_rng, Rng};

use crate::filter::get_filtered_urls;

use log::{error, info};

use crate::searx_client::SearxClient;

use crate::Cache;

use std::sync::Mutex;

use actix_web::web::Data;

pub fn get_random_url_from_cache(cache: &Data<Mutex<Cache>>) -> String {
    let cache_guard = cache.lock().unwrap();
    let instance_urls = &cache_guard.instances;
    get_random_instance_url(instance_urls)
}

pub(crate) async fn populate_cache_if_needed(
    cache: &Data<Mutex<Cache>>,
    client: &Data<SearxClient>,
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
    match cache.creation_time.elapsed() {
        Ok(elapsed) => elapsed.as_secs() > cache.ttl.as_secs(),
        Err(e) => {
            error!("Error: {e:?}");
            true
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::HOUR;
    use core::time::Duration;
    use std::time::SystemTime;

    use super::*;
    #[test]
    fn ttl_exceeded_test() {
        let creation_time = SystemTime::now();
        let cache = Cache {
            creation_time,
            instances: Vec::new(),
            ttl: Duration::from_secs(HOUR.into()),
        };
        assert!(!ttl_exceeded(&cache));

        let creation_time = SystemTime::now();
        let cache = Cache {
            creation_time,
            instances: Vec::new(),
            ttl: Duration::from_secs(25),
        };
        assert!(!ttl_exceeded(&cache));

        let creation_time = creation_time
            .checked_add(Duration::from_secs((HOUR + 200).into()))
            .unwrap();
        let cache = Cache {
            creation_time,
            instances: Vec::new(),
            ttl: Duration::from_secs(HOUR.into()),
        };
        assert!(ttl_exceeded(&cache));

        let creation_time = creation_time
            .checked_add(Duration::from_secs(2 + 2))
            .unwrap();
        let cache = Cache {
            creation_time,
            instances: Vec::new(),
            ttl: Duration::from_secs(2),
        };
        assert!(ttl_exceeded(&cache));
    }
}