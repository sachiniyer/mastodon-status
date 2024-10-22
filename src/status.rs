use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use url::Url;

/// The StatusResponse enum is used to represent the status of the services
///
/// # Variants
/// - Broken: The k3s cluster is down and no requests are able to be made
/// - Degraded: Some services are down, but not all
/// - Passing: All services are up and running
#[derive(Debug)]
pub enum StatusResponse {
    Broken,
    Degraded(HashMap<String, bool>),
    Passing(HashMap<String, bool>),
}

/// Implement the PartialEq trait for the StatusResponse enum
///
/// Broken + Broken = true
/// Degraded + Degraded = all services share the same status
/// Passing + Passing = all services are the same
/// Otherwise, the two StatusResponses are not equal
///
/// # Arguments
/// - self: The first StatusResponse to compare
/// - other: The second StatusResponse to compare
///
/// # Returns
/// - true if the two StatusResponses are equal, false otherwise
impl PartialEq for StatusResponse {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StatusResponse::Broken, StatusResponse::Broken) => true,
            (StatusResponse::Degraded(i), StatusResponse::Degraded(j)) => {
                if i.len() != j.len() {
                    return false;
                }
                for (k, v) in i {
                    if !j.contains_key(k) {
                        return false;
                    }
                    if j.get(k).unwrap() != v {}
                }
                true
            }
            (StatusResponse::Passing(_), StatusResponse::Passing(_)) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

/// Implement the Display trait for the StatusResponse enum
///
/// # Examples
///
/// ```md
/// Degraded
/// playground: t
/// invoice: t
/// digits-api: t
/// wiki: t
/// bin: t
/// digits: t
/// tweets: t
/// blog: t
/// meet: t
/// crabfit-api: t
/// sachiniyer: t
/// resow: t
/// rss: t
/// emptypad: t
/// sembox: t
/// resow-api: t
/// s: t
/// invoice-api: t
/// school-demo: t
/// share: t
/// git: t
/// sembox-api: f
/// ```
///
/// # Arguments
/// - self: The StatusResponse to display
/// - f: The Formatter to write to
///
/// # Returns
/// - fmt::Result
impl Display for StatusResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StatusResponse::Broken => write!(f, "Broken"),
            StatusResponse::Degraded(map) => {
                writeln!(f, "Degraded")?;
                for (k, v) in map {
                    writeln!(
                        f,
                        "{}: {}",
                        k,
                        match v {
                            true => "t",
                            false => "f",
                        }
                    )?;
                }
                Ok(())
            }
            StatusResponse::Passing(map) => {
                writeln!(f, "Passing")?;
                for (k, v) in map {
                    writeln!(
                        f,
                        "{}: {}",
                        k,
                        match v {
                            true => "t",
                            false => "f",
                        }
                    )?;
                }
                Ok(())
            }
        }
    }
}

/// Implement the into_map function for the StatusResponse enum
///
/// Does the following:
/// 1. Reads the html with `html2text`
/// 2. Logs the parsed html
/// 3. Constructs a new result HashMap
/// 4. Iterates over the parsed html
///   - Splits the line at the last colon
///   - Trims the key and value
///   - Inserts the key and value into the HashMap
/// 5. Returns the HashMap
///
/// # Arguments
/// - s: The string to parse
///
/// # Returns
/// - A Result containing the HashMap of services and their status or ()
impl StatusResponse {
    fn into_map(s: &str) -> Result<HashMap<String, bool>, ()> {
        let decorator = html2text::render::text_renderer::TrivialDecorator::new();
        let parsed = html2text::from_read_with_decorator(s.as_bytes(), 8192, decorator);
        let mut map = HashMap::new();
        println!("{}", parsed.to_string());
        for line in parsed.lines().skip(1) {
            let (key, value) = match line.rfind(":") {
                Some(index) => {
                    let (first, second) = line.split_at(index);
                    let second = &second[1..];
                    (first, second)
                }
                None => {
                    panic!("Something went wrong when parsing StatusResponse");
                }
            };
            let value = match value.trim() {
                "t" => true,
                "f" => false,
                _ => panic!("Invalid true or false value when parsing StatusResponse"),
            };
            println!("{},{}", key, value);
            map.insert(key.trim().to_string(), value);
        }
        Ok(map)
    }
}

/// Implement the FromStr trait for the StatusResponse enum
///
/// # Arguments
/// - s: The string to parse
///
/// # Returns
/// - A Result containing the StatusResponse or ()
impl FromStr for StatusResponse {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("Broken") {
            return Ok(StatusResponse::Broken);
        }
        if s.contains("Degraded") {
            return Ok(StatusResponse::Degraded(Self::into_map(s)?));
        }
        if s.contains("Passing") {
            return Ok(StatusResponse::Passing(Self::into_map(s)?));
        }
        Err(())
    }
}

/// Truncate the message to only include the first component of the URL
///
/// # Examples
/// ```md
/// https://playground.sachiniyer.com => playground
/// ```
///
/// # Arguments
/// - map: The HashMap of services and their status to truncate
///
/// # Returns
/// - A truncated HashMap of services and their status
fn truncate_message(map: HashMap<String, bool>) -> HashMap<String, bool> {
    map.into_iter()
        .map(|(k, v)| {
            let url = Url::parse(&k).unwrap();
            let authority = url.host_str().unwrap();
            let components: Vec<&str> = authority.split('.').collect();
            let first_component = components.first().unwrap();
            (first_component.to_string(), v)
        })
        .collect()
}

/// Get the status of the services
///
/// Does the following:
/// 1. Get the STATUS_API environment variable
/// 2. Send a GET request to the URL
/// 3. Constructs a HashMap of the services and their status
/// 4. Removes the computer.sachiniyer.com service (goes up and down whenever I open laptop)
/// 5. Constructs a Vec of services that are down
/// 6. Returns a StatusResponse based on the number of services that are down
/// 7. If the request fails, return a Broken StatusResponse
///
/// # Returns
/// - A Result containing the StatusResponse or a reqwest::Error
pub async fn get_status() -> Result<StatusResponse, reqwest::Error> {
    let vars = env::vars().collect::<HashMap<String, String>>();
    let url = vars.get("STATUS_API").unwrap();

    let client = Client::new();
    if let Ok(res) = client.get(url).send().await {
        if let StatusCode::OK = res.status() {
            let json = res.json::<Value>().await?;
            let mut services: HashMap<String, bool> = HashMap::new();
            for i in json.as_array().unwrap() {
                services.insert(
                    i.get("site").unwrap().as_str().unwrap().to_string(),
                    i.get("status").unwrap().as_bool().unwrap(),
                );
            }
            // this fluctuates too much, so just remove it.
            services.remove("https://computer.sachiniyer.com");
            let mut degraded = Vec::new();
            for (k, v) in &services {
                if !v {
                    degraded.push((k.to_string(), *v));
                }
            }
            match degraded.len() {
                0 => return Ok(StatusResponse::Passing(truncate_message(services))),
                _ => return Ok(StatusResponse::Degraded(truncate_message(services))),
            }
        }
    };
    Ok(StatusResponse::Broken)
}
