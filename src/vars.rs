use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

/// Collects all environment variables and checks if all required variables are set
///
/// # Returns
/// - None if all required environment variables are set
/// - Some(String) if any required environment variable is not set
fn check_envs() -> Option<String> {
    let var_map: HashMap<String, String> = env::vars().collect();
    let mut vars: HashMap<String, Option<&String>> = HashMap::new();
    let envs = vec!["INSTANCE_URL", "ACCESS_TOKEN", "STATUS_API"];
    envs.iter().for_each(|e| {
        vars.insert(e.to_string(), var_map.get(&e.to_string()));
    });
    check_vars(vars)
}

/// Given a hashmap of variables, check if any of them are None
///
/// # Arguments
/// - vars: A hashmap of Option variables to check
///
/// # Returns
/// - None if all variables are present
/// - Some(String) if any variable is not present
fn check_vars<T>(vars: HashMap<String, Option<T>>) -> Option<String> {
    if vars.values().any(|v| v.is_none()) {
        return Some(
            vars.iter()
                .enumerate()
                .filter(|(_, v)| v.1.is_none())
                .fold("".to_string(), |p, c| format!("{} {}", p, c.1 .0)),
        );
    };
    None
}

/// Check if all required environment variables are set
///
/// # Returns
/// - None if all required environment variables are set
/// - Some(Value) if any required environment variable is not set
pub fn check() -> Option<Value> {
    let mut res = String::new();
    if let Some(v) = check_envs() {
        res.push_str(format!("envs: {} not given ", v).as_str());
    }
    if res != "" {
        return Some(json!({ "error": res }));
    }
    None
}
