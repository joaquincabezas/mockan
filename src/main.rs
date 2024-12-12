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
                for service in services 
                    {
                        println!("Api path: {}", service.path);
                        println!("Delay: {}", service.delay);
                    }

                    let routes = warp::any().map(|| "Hello, World!");

                    warp::serve(routes).run(([127, 0, 0, 1], default_port)).await;
            }
            Err(code) => {
                // Handle the Err case (i.e., the error)
                println!("Error code: {}", code);
            }
        
        }

        }
        Err(..) => {}
    }

    println!("Nothing!")

}

