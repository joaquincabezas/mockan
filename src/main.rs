mod readapi;

fn main() {
    // Load the OpenAPI service map
    let api_spec = readapi::get_spec("config/example.yaml");

    match api_spec {
        Ok(spec) => {

        let _default_port = readapi::get_default_port(&spec);
        let api_services = readapi::get_paths(&spec);

        match api_services {
            Ok(services) => {
                for service in services 
                    {
                        println!("Api path: {}", service.path);
                        println!("Delay: {}", service.delay);
                    }
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

