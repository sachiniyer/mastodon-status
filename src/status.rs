use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use url::Url;

#[derive(Debug)]
pub enum StatusResponse {
    Broken,
    Degraded(HashMap<String, bool>),
    Passing(HashMap<String, bool>),
}

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

impl StatusResponse {
    fn into_map(s: &str) -> Result<HashMap<String, bool>, ()> {
        let decorator = html2text::render::text_renderer::TrivialDecorator::new();
        let parsed = html2text::from_read_with_decorator(s.as_bytes(), 8192, decorator);
        let mut map = HashMap::new();
        println!("{:?}", parsed);
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
