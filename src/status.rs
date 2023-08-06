use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

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
                for (ii, ij) in i.iter() {
                    if !j.contains_key(ii) || ij != j.get(ii).unwrap() {
                        return false;
                    }
                }
                for (ji, jj) in j.iter() {
                    if !i.contains_key(ji) || jj != i.get(ji).unwrap() {
                        return false;
                    }
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
                    writeln!(f, "{}: {}", k, v)?;
                }
                Ok(())
            }
            StatusResponse::Passing(map) => {
                writeln!(f, "Passing")?;
                for (k, v) in map {
                    writeln!(f, "{}: {}", k, v)?;
                }
                Ok(())
            }
        }
    }
}

impl FromStr for StatusResponse {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("Broken") {
            return Ok(StatusResponse::Broken);
        }
        if s.contains("Degraded") {
            let mut map = HashMap::new();
            for line in s.lines().skip(1) {
                let mut split = line.split(": ");
                let k = split.next().unwrap();
                let v = split.next().unwrap();
                map.insert(k.to_string(), v.parse::<bool>().unwrap());
            }
            return Ok(StatusResponse::Degraded(map));
        }
        if s.contains("Passing") {
            let mut map = HashMap::new();
            for line in s.lines().skip(1) {
                let mut split = line.split(": ");
                let k = split.next().unwrap();
                let v = split.next().unwrap();
                map.insert(k.to_string(), v.parse::<bool>().unwrap());
            }
            return Ok(StatusResponse::Passing(map));
        }
        Err(())
    }
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
                    i["site"].as_str().unwrap().to_string(),
                    i["status"].as_bool().unwrap(),
                );
            }

            let mut degraded = Vec::new();
            for (k, v) in &services {
                if !v {
                    degraded.push((k.to_string(), *v));
                }
            }
            match degraded.len() {
                0 => return Ok(StatusResponse::Passing(services)),
                _ => return Ok(StatusResponse::Degraded(services)),
            }
        }
    };
    Ok(StatusResponse::Broken)
}
