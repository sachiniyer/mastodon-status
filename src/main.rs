use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

mod mastodon;
mod pagerduty;
mod status;
mod vars;

/// Lambda function entrypoint
#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    let res = lambda_runtime::run(func).await;
    if res.is_err() {
        println!("here");
    }
    Ok(())
}

/// Lambda function
///
/// It does the following
/// 1. Check if the environment variables are set
/// 2. Login to Mastodon
/// 3. Get the previous status
/// 4. Get the current status from k3s cluster
/// 5. If the status is different, send a new status to Mastodon
/// 6. Return the status
/// 7. Log the results
async fn func(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    let result = async {
        if let Some(v) = crate::vars::check() {
            return Ok(v);
        }

        let (client, id) = mastodon::login().await?;

        let prev_status: status::StatusResponse =
            match mastodon::get_post(&client, &id, None).await?.parse() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error parsing previous status: {:?}", e);
                    return Err(Error::from("Error parsing previous status"));
                }
            };

        let status = status::get_status().await?;

        let mut changed = false;
        if prev_status != status {
            mastodon::send_post(&client, status.to_string()).await?;
            pagerduty::send_page(Some(status.to_string())).await;
            changed = true;
        };

        let res = json!({ "message": status.to_string(), "changed": changed});
        println!(
            "Previous: {}\nCurrent: {}\nDifferent: {}\nResult: {}",
            prev_status.to_string(),
            status.to_string(),
            prev_status != status,
            res.to_string()
        );
        Ok(res)
    }
    .await;
    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
        pagerduty::send_page(None).await;
        return Err(Error::from("Error in the lambda function"));
    }
    result
}
