use chrono::{DateTime, Duration, Utc};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

mod mastodon;
mod pagerduty;
mod status;
mod vars;

/// Lambda function entrypoint
#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

/// Lambda function
///
/// It does the following
/// 1. Check if the environment variables are set
/// 2. Login to Mastodon
/// 3. Get the previous status/publishing time
/// 4. Get the current status from k3s cluster
/// 5. If the status is different and cooldown period has passed, send a new status to Mastodon
/// 6. Return the status
/// 7. Log the results
async fn func(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    let result = async {
        if let Some(v) = crate::vars::check() {
            return Ok(v);
        }

        let vars = env::vars().collect::<HashMap<String, String>>();

        let (client, id) = mastodon::login().await?;

        let (prev_status, prev_status_time): (status::StatusResponse, DateTime<Utc>) =
            match mastodon::get_post(&client, &id, None).await {
                Ok(s) => {
                    let prev_status_time = s.1;
                    let prev_status = match s.0.parse() {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error parsing previous status: {:?}", e);
                            return Err(Error::from("Error parsing previous status"));
                        }
                    };
                    (prev_status, prev_status_time)
                }
                Err(e) => {
                    eprintln!("Error retrieving previous status: {:?}", e);
                    return Err(Error::from("Error retrieving previous status"));
                }
            };

        let status = status::get_status().await?;
        let status_time = Utc::now();
        let cooldown_time = Duration::seconds(
            vars.get("POST_COOLDOWN_SECS")
                .unwrap()
                .clone()
                .parse::<i64>()
                .unwrap(),
        );

        let mut changed = false;
        if prev_status != status
            && status_time.signed_duration_since(prev_status_time) > cooldown_time
        {
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
        return Err(Error::from(format!(
            "Error in the lambda function , {:?}",
            e
        )));
    }
    result
}
