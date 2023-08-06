use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

mod mastodon;
mod status;
mod vars;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    if let Some(v) = crate::vars::check() {
        return Ok(v);
    }

    let (client, id) = mastodon::login().await.unwrap();
    let prev_status = mastodon::get_post(&client, id).await.unwrap();
    let prev_status: status::StatusResponse = prev_status.parse().unwrap();
    let status = status::get_status().await.unwrap();
    println!("status {}", status);
    if prev_status != status {
        mastodon::send_post(&client, status.to_string())
            .await
            .unwrap();
        return Ok(json!({ "message": format!("Status {}", status),
                           "changed": true}));
    }

    let res = json!({ "message": format!("Status {}", status),
                "changed": false});
    Ok(res)
}
