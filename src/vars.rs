use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

fn check_envs() -> Option<String> {
    let var_map: HashMap<String, String> = env::vars().collect();
    let mut vars: HashMap<String, Option<&String>> = HashMap::new();
    let envs = vec!["INSTANCE_URL", "ACCESS_TOKEN", "STATUS_API"];
    envs.iter().for_each(|e| {
        vars.insert(e.to_string(), var_map.get(&e.to_string()));
    });
    check_vars(vars)
}

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
