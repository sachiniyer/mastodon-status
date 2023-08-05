use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

mod mastodon;
mod vars;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();

    if let Some(v) = crate::vars::check(event.clone()) {
        return Ok(v);
    }
    let (client, id) = mastodon::login().await.unwrap();
    let prev_status = mastodon::get_post(&client, id).await.unwrap();
    let status = event["status"].as_bool().unwrap_or(false);
    if prev_status != status {
        mastodon::send_post(&client, status).await.unwrap();
        return Ok(json!({ "message": format!("Status {}", status),
                           "changed": true}));
    }

    Ok(json!({ "message": format!("Status {}", status),
                "changed": false}))
}
