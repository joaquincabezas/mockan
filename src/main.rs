mod readapi;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Load the OpenAPI service map
    let api_spec = readapi::get_spec("config/example.yaml");

    match api_spec {
        Ok(spec) => {
            let default_port = readapi::get_default_port(&spec);
            let api_services = readapi::get_paths(&spec);

            match api_services {
                Ok(services) => {
                    if services.is_empty() {
                        eprintln!("No services found in the API specification.");
                        return;
                    }

                    let hello = warp::path("hello").map(|| "Hello, World!").boxed();

                    let mut routes = hello;

                    for service in services {
                        let path = service.path.clone();
                        let route = warp::path(path).map(|| "Hello, World!").boxed();
                        routes = routes.or(route).unify().boxed();
                    }

                    // Serve the combined routes
                    warp::serve(routes)
                        .run(([127, 0, 0, 1], default_port))
                        .await;
                }
                Err(code) => {
                    // Handle the error case when retrieving API services
                    eprintln!("Error retrieving API services: {}", code);
                }
            }
        }
        Err(err) => {
            // Handle the error case when loading the API specification
            eprintln!("Error loading API specification: {:?}", err);
        }
    }

    println!("Server has stopped.");
}
