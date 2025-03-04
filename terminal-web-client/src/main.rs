use clap::{Arg, ArgAction, Command};
use reqwest::{Client, Error, Method};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse command-line arguments using clap
    let matches = Command::new("Rust cURL")
        .version("1.0")
        .author("rclid780 <youremail@example.com>")
        .about("Rust equivalent of cURL")
        .arg(Arg::new("url")
            .help("The URL to make the request to")
            .required(true)
            .index(1))
        .arg(Arg::new("method")
            .help("The HTTP method (GET, POST, etc.)")
            .required(true)
            .short('X')
            .long("method"))
        .arg(Arg::new("headers")
            .help("The headers to include in the request, in key:value format")
            .long("headers")
            .action(ArgAction::Append))
        .arg(Arg::new("body")
            .help("The body of the request (for POST, PUT, etc.)")
            .long("body"))
        .get_matches();

    let url = matches.get_one::<String>("url").unwrap(); // URL to request

    let method_str = matches.get_one::<String>("method").unwrap(); // HTTP method (GET, POST, etc.)
    
    // Parse headers if any are provided
    let mut headers = HashMap::new();
    if let Some(header_values) = matches.get_many::<String>("headers") {
        for header in header_values.collect::<Vec<_>>() {
            let mut splitter = header.splitn(2, ":");
            
            if let Some(first) = splitter.next() {
                if let Some(second) = splitter.next() {
                    headers.insert(first.trim().to_string(), second.trim().to_string());
                }
                else {
                    eprintln!("Header format should be \"key:value\", found \"{}\"", header);
                    return Ok(());                
                }
            }
        }
    };
    
    // Parse body if provided
    let body_str = matches.get_one::<String>("body");

    // Create the HTTP client
    let client = Client::new();

    // Convert the string method to an actual Method enum
    let method = match method_str.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        _ => {
            eprintln!("Unsupported HTTP method: {}", method_str);
            return Ok(());
        }
    };

    // Start building the request
    let mut request = client.request(method, url);

    // Add headers to the request if there are any
    for (key, value) in headers {
        request = request.header(key, value);
    }

    // Add the body to the request if provided (for POST, PUT, etc.)
    if let Some(body) = body_str {
        request = request.body(body.to_string());
    }

    // Send the request
    let response = request.send().await?;

    // Check the response status
    if response.status().is_success() {
        let response_body = response.text().await?;
        println!("Response: {}", response_body);
    } else {
        eprintln!("Request failed with status: {}", response.status());
    }

    Ok(())
}
