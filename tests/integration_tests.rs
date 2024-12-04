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
    response_file: String,
}

#[tokio::test]
async fn test_valid_service_response() {
    // Setup
    let config = load_config("config.yaml");
    let service_map = load_service_map(&config);
    let routes = build_routes(config, service_map);

    // Test for /v2/models/example/infer
    let response = warp::test::request()
        .method("GET")
        .path("/v2/models/example/infer")
        .reply(&routes)
        .await;

    assert_eq!(response.status(), 200);

    let body: serde_json::Value =
        serde_json::from_slice(response.body()).expect("Invalid JSON response");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["prediction"], serde_json::json!([0.1, 0.9]));
}

#[tokio::test]
async fn test_valid_service_response_with_delay() {
    let config = load_config("config.yaml");
    let service_map = load_service_map(&config);
    let routes = build_routes(config, service_map);

    // Start time
    let start = tokio::time::Instant::now();

    // Test for /v2/models/other-example/infer
    let response = warp::test::request()
        .method("GET")
        .path("/v2/models/other-example/infer")
        .reply(&routes)
        .await;

    // Check if response is delayed properly
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() >= 800);

    assert_eq!(response.status(), 200);

    let body: serde_json::Value =
        serde_json::from_slice(response.body()).expect("Invalid JSON response");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["prediction"], serde_json::json!([0.1, 0.9]));
}

#[tokio::test]
async fn test_404_for_unknown_path() {
    let config = load_config("config.yaml");
    let service_map = load_service_map(&config);
    let routes = build_routes(config, service_map);

    // Test for an undefined path
    let response = warp::test::request()
        .method("GET")
        .path("/v2/models/unknown/infer")
        .reply(&routes)
        .await;

    assert_eq!(response.status(), 404);
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
            let response_content = fs::read_to_string(&service.response_file)
                .expect(&format!("Unable to load response file: {}", service.response_file));
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
    // Get the full request path
    let request_path = path.as_str().to_string();

    // Find the service configuration based on the path
    if let Some(service) = config.services.values().find(|s| s.path == request_path) {
        // Apply the service-specific delay
        let delay = service.delay;
        sleep(Duration::from_millis(delay)).await;

        // Return the JSON response from the service map
        if let Some(response) = service_map.get(&service.path) {
            let json_response = serde_json::from_str::<serde_json::Value>(response).unwrap();
            return Ok(warp::reply::json(&json_response));
        }
    }

    // If no service is found, return a 404 error
    Err(warp::reject::not_found())
}
