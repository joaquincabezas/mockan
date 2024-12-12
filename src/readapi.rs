use oas3::{from_path};

#[derive(Clone)]
pub struct ServiceConfig {
    pub path: String,
    pub delay: u64,
}

pub fn get_spec(api_file: &str) -> Result<oas3::Spec, oas3::Error> {
    from_path(api_file)
}

pub fn get_paths(api_spec: &oas3::Spec) -> Result<Vec<ServiceConfig>, u64> {

    let mut api_paths: Vec<ServiceConfig> = Vec::new();

    if let Some(paths) = &api_spec.paths {

        for (path, path_item) in paths {

            let get_item = path_item.get.clone().unwrap(); // Create a longer-lived variable
            let delay_op = get_item.extensions.get("delay-ms");

            let delay_value = match delay_op {
                Some(v) => v.as_i64().unwrap_or(0) as u64, // Default to 0 if invalid
                None => 0,                                 // Default to 0 if None
            };

            let new_service = ServiceConfig { path:path.trim_start_matches('/').to_string(), delay: delay_value };

            api_paths.push(new_service);
        }
    }
    else
    {
        return Err(0);
    }

    return Ok(api_paths);
}


pub fn get_default_port(api_spec: &oas3::Spec) -> u16 {
    if api_spec.servers.len() > 1 {
        println!("Currently mockan only supports one server in the OpenAPI specification");
        return 0;
    }
    for server in &api_spec.servers {
        let port_obj = server.variables.get("port");

        match &port_obj {
            Some(port) => {
                let default_port: u16 = port.default.parse().expect("Default port must be a number");
                return default_port;
            }
            None => {
                return 0;
            }
        }
    }
    0
}
