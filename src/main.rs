use warp::Filter;
use serde::Deserialize;
use std::{fs, sync::Arc};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct Config {
    services: HashMap<String, ServiceConfig>,
}

#[derive(Debug, Deserialize)]
struct ServiceConfig {
    path: String,
    delay: u64,
    response: String,
}

#[tokio::main]
async fn main() {
    let config = load_config("config/services.yaml");
    let service_map = load_service_map(&config);
    let routes = build_routes(config, service_map);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

fn load_config(path: &str) -> Arc<Config> {
    let config: Config = serde_yaml::from_str(
        &fs::read_to_string(path).expect(&format!("Unable to load {}", path)),
    )
    .expect("Failed to parse config.yaml");
    Arc::new(config)
}

fn load_service_map(config: &Arc<Config>) -> Arc<HashMap<String, Arc<String>>> {
    let service_map: HashMap<String, Arc<String>> = config
        .services
        .iter()
        .map(|(_, service)| {
            let response_content = fs::read_to_string(&service.response)
                .expect(&format!("Unable to load response file: {}", service.response));
            (service.path.clone(), Arc::new(response_content))
        })
        .collect();
    Arc::new(service_map)
}

fn build_routes(
    config: Arc<Config>,
    service_map: Arc<HashMap<String, Arc<String>>>,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    warp::path::full()
        .and(warp::get())
        .and(with_config(config.clone()))
        .and(with_service_map(service_map.clone()))
        .and_then(handle_request)
        .boxed()
}

fn with_config(
    config: Arc<Config>,
) -> impl Filter<Extract = (Arc<Config>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_service_map(
    service_map: Arc<HashMap<String, Arc<String>>>,
) -> impl Filter<Extract = (Arc<HashMap<String, Arc<String>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || service_map.clone())
}

async fn handle_request(
    path: warp::filters::path::FullPath,
    config: Arc<Config>,
    service_map: Arc<HashMap<String, Arc<String>>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let request_path = path.as_str().to_string();

    if let Some(service) = config.services.values().find(|s| s.path == request_path) {
        let delay = service.delay;
        sleep(Duration::from_millis(delay)).await;

        if let Some(response) = service_map.get(&service.path) {
            let json_response = serde_json::from_str::<serde_json::Value>(response).unwrap();
            return Ok(warp::reply::json(&json_response));
        }
    }

    Err(warp::reject::not_found())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = load_config("config/services.yaml");
        assert_eq!(config.services.len(), 3);

        let example_service = config.services.get("example").unwrap();
        assert_eq!(example_service.path, "/v2/models/example/infer");
        assert_eq!(example_service.delay, 1200);
        assert_eq!(example_service.response, "config/response.json");
    }

    #[test]
    fn test_load_service_map() {
        let config = load_config("config/services.yaml");
        let service_map = load_service_map(&config);

        assert!(service_map.contains_key("/v2/models/example/infer"));
        assert!(service_map.contains_key("/v2/models/other-example/infer"));

        let example_response = service_map.get("/v2/models/example/infer").unwrap();
        let response_body: serde_json::Value =
            serde_json::from_str(example_response).expect("Invalid JSON response");
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["prediction"], serde_json::json!([0.1, 0.9]));
    }

    #[tokio::test]
    async fn test_handle_request_valid_path() {
        let config = load_config("config/services.yaml");
        let service_map = load_service_map(&config);

        let request_path = warp::test::request()
            .path("/v2/models/example/infer")
            .reply(&build_routes(config.clone(), service_map.clone()))
            .await;

        assert_eq!(request_path.status(), 200);
    }

    #[tokio::test]
    async fn test_handle_request_invalid_path() {
        let config = load_config("config/services.yaml");
        let service_map = load_service_map(&config);

        let response = warp::test::request()
            .path("/v2/models/unknown/infer")
            .reply(&build_routes(config.clone(), service_map.clone()))
            .await;

        assert_eq!(response.status(), 404);
    }

    #[tokio::test]
    async fn test_delay() {
        let config = load_config("config/services.yaml");
        let service_map = load_service_map(&config);

        let start_time = tokio::time::Instant::now();

        warp::test::request()
            .path("/v2/models/example/infer")
            .reply(&build_routes(config.clone(), service_map.clone()))
            .await;

        let elapsed = start_time.elapsed().as_millis();
        assert!(elapsed >= 1200);
    }
}
