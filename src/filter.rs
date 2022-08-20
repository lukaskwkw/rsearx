use log::trace;
use serde_json::{Map, Value};

pub fn get_filtered_urls(instances: &Map<String, Value>) -> Vec<&String> {
    let best_grade_instance_urls: Vec<&String> = instances
        .iter()
        .filter_map(|k| {
            let grade: String = k.1["html"]["grade"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let network_type: String = k.1["network_type"].as_str().unwrap_or_default().to_string();
            trace!("grade {grade}, network_type {network_type}");
            if ["C", "V"].contains(&&grade[..]) && network_type == "normal" {
                Some(k.0)
            } else {
                None
            }
        })
        .collect();
    best_grade_instance_urls
}
