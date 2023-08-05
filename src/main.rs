use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    let status = event["status"].as_bool().unwrap_or(false);
    let token = event["token"].as_str().unwrap_or("token");

    Ok(json!({
        "message": format!("Status {} and Token {}", status, token)
    }))
}
