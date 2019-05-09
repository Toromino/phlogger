extern crate config;

use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind(format!(
        "{domain}:{port}",
        domain = get_config_value("endpoint.address"),
        port = get_config_value("endpoint.port")
    ))
    .unwrap_or_else(|_| panic!("Listener could not be started!"));

    for connection in listener.incoming() {
        let mut handler = connection.unwrap();
        let mut buffer = [0; 2048];

        handler.read(&mut buffer).unwrap();
        let mut route = String::from_utf8_lossy(&buffer[..]).to_string();
        route = route[1..]
            .replace("\r", "")
            .replace("\n", "")
            .replace(" ", "")
            .trim_matches(char::from(0))
            .to_string();

        if route == "".to_string() {
            route = String::from("index")
        }

        let file = match std::fs::read_to_string(format!("static/public/{}", &route)) {
            Ok(body) => {
                println!(
                    "Matched route /{uri} with file 'static/public/{uri}'",
                    uri = &route
                );
                body
            }
            Err(_) => std::fs::read_to_string(format!("static/public/{}/index", &route))
                .unwrap_or_else(|_| String::from("Error: Page not found")),
        };

        let mut response: String = String::new();
        for line in file.lines() {
            let mut body: String = line.to_string();

            if line.chars().next().unwrap() == '0' || line.chars().next().unwrap() == '1' {
                body = line.replacen("/", "\t/", 1);
            }

            response = format!(
                "{ancestors}{body}\t{address}\t{port}\r\n",
                ancestors = response,
                body = body,
                address = get_config_value("endpoint.address"),
                port = get_config_value("endpoint.port")
            );
        }

        handler.write(response.as_bytes()).unwrap();
        handler.flush().unwrap();
    }
}

fn get_config_value(key: &str) -> String {
    let mut config = config::Config::default();

    config.merge(config::File::with_name("env.toml"));

    match config.get_str(key) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Config value '{}' not found", key);
            return String::from("");
        }
    }
}
