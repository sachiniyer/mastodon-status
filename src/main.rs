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

    let prev_status: status::StatusResponse = mastodon::get_post(&client, id)
        .await
        .unwrap()
        .parse()
        .unwrap();

    let status = status::get_status().await.unwrap();

    let mut changed = false;
    if prev_status != status {
        mastodon::send_post(&client, status.to_string())
            .await
            .unwrap();
        changed = true;
    };

    let res = json!({ "message": status.to_string(), "changed": changed});
    println!("Previous: {}", prev_status.to_string());
    println!("Current: {}", status.to_string());
    println!("Different: {}", prev_status != status);
    println!("{}", res.to_string());
    Ok(res)
}
