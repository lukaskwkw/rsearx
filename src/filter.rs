use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Timings {
    pub search: Option<f32>,
    pub google: Option<f32>,
    pub wikipedia: Option<f32>,
    pub initial: Option<f32>,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Filter {
    pub response_times: Option<Timings>,
    pub grades: Option<Vec<String>>,
    pub versions: Option<(String, String)>,
}

pub fn get_filtered_urls<'a>(
    instances: &'a Map<String, Value>,
    filter: &'a Filter,
) -> Vec<&'a String> {
    let best_grade_instance_urls: Vec<&String> = instances
        .iter()
        .filter_map(|instance| {
            // trace!("grade {grade}, network_type {network_type}");
            if filter_by_grade(instance, filter)
                && filter_by_timings(instance, filter)
                && filter_by_network(instance)
            {
                Some(instance.0)
            } else {
                None
            }
        })
        .collect();
    best_grade_instance_urls
}

type Instance<'a> = (&'a String, &'a Value);
fn filter_by_grade<'a>(instance: Instance, filter: &'a Filter) -> bool {
    let (_url, value) = instance;

    let grade: String = value["html"]["grade"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    filter
        .grades
        .clone()
        .unwrap_or_else(|| vec!["C".to_string(), "V".to_string()])
        .contains(&grade)
}

fn filter_by_network<'a>(instance: Instance) -> bool {
    let (_url, value) = instance;
    let network_type: String = value["network_type"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    network_type == "normal"
}

fn filter_by_timings<'a>(instance: Instance, filter: &'a Filter) -> bool {
    let response_times = match filter.response_times.clone() {
        Some(times) => times,
        None => return true,
    };
    let (_url, value) = instance;
    let timing_obj: &Value = &value["timing"];
    let mut is_initial_search_ok = true;
    let mut is_search_ok = true;
    let mut is_search_go_ok = true;
    let mut is_search_wp_ok = true;
    if let Some(timing) = response_times.search {
        is_search_ok = timing_obj["search"]["all"]["mean"]
            .as_f64()
            .map(|mean| mean < timing as f64)
            .unwrap_or_default();
    }
    if let Some(timing) = response_times.google {
        is_search_go_ok = timing_obj["search_go"]["all"]["mean"]
            .as_f64()
            .map(|mean| mean < timing as f64)
            .unwrap_or_default();
    }
    if let Some(timing) = response_times.wikipedia {
        is_search_wp_ok = timing_obj["search_wp"]["all"]["mean"]
            .as_f64()
            .map(|mean| mean < timing as f64)
            .unwrap_or_default();
    }
    if let Some(timing) = response_times.initial {
        is_initial_search_ok = timing_obj["initial"]["all"]["mean"]
            .as_f64()
            .map(|mean| mean < timing as f64)
            .unwrap_or_default();
    }
    is_initial_search_ok && is_search_ok && is_search_go_ok && is_search_wp_ok
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::json;

    use super::*;
    #[test]
    fn filter_by_timings_test() {
        let json = json!({
        "timing": {
            "search": {
                "all": {
                    "mean": 0.52
                }
            },
            "search_go": {
                "all": {
                    "mean": 0.52
                }
            },
            "search_wp": {
                "all": {
                    "mean": 0.52
                }
            },
            "initial": {
                "all": {
                    "mean": 0.52
                }
            },
        }
        });
        let filter = Filter {
            response_times: Some(Timings {
                search: Some(0.4f32),
                google: Some(0.4f32),
                wikipedia: Some(0.4f32),
                initial: Some(0.4f32),
            }),
            ..Filter::default()
        };
        let url = "url".to_string();
        let instance = (&url, &json);
        let include = filter_by_timings(instance, &filter);
        assert!(!include);

        let filter = Filter {
            response_times: Some(Timings {
                search: None,
                google: Some(0.4f32),
                wikipedia: Some(0.4f32),
                initial: Some(0.4f32),
            }),
            ..Filter::default()
        };
        let url = "url".to_string();
        let instance = (&url, &json);
        let include = filter_by_timings(instance, &filter);
        assert!(!include);

        let filter = Filter {
            response_times: Some(Timings {
                search: Some(0.6f32),
                google: Some(0.4f32),
                wikipedia: Some(0.4f32),
                initial: Some(0.4f32),
            }),
            ..Filter::default()
        };
        let url = "url".to_string();
        let instance = (&url, &json);
        let include = filter_by_timings(instance, &filter);
        assert!(!include);

        let filter = Filter {
            response_times: Some(Timings {
                search: Some(0.6f32),
                google: Some(0.6f32),
                wikipedia: Some(0.6f32),
                initial: Some(0.6f32),
            }),
            ..Filter::default()
        };
        let url = "url".to_string();
        let instance = (&url, &json);
        let include = filter_by_timings(instance, &filter);
        assert!(include);

        let filter = Filter {
            response_times: None,
            ..Filter::default()
        };
        let url = "url".to_string();
        let instance = (&url, &json);
        let include = filter_by_timings(instance, &filter);
        assert!(include);
    }
    #[test]
    fn filter_by_timings_test_json_bad() {
        let json = json!({
        "wrong": {
            "nothing": "here"
        }
        });
        let filter = Filter {
            response_times: Some(Timings {
                search: Some(0.4f32),
                google: Some(0.4f32),
                wikipedia: Some(0.4f32),
                initial: Some(0.4f32),
            }),
            ..Filter::default()
        };
        let url = "url".to_string();
        let instance = (&url, &json);
        let include = filter_by_timings(instance, &filter);
        assert!(!include);

        let json = json!({
        "wrong": {
            "nothing": "here"
        }
        });
        let filter = Filter {
            response_times: None,
            ..Filter::default()
        };
        let url = "url".to_string();
        let instance = (&url, &json);
        let include = filter_by_timings(instance, &filter);
        assert!(include);
    }
}
