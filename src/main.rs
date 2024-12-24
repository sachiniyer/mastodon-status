use lambda_runtime::{service_fn, Error, LambdaEvent};
use pagerduty_rs::eventsv2sync::*;
use pagerduty_rs::types::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use time::OffsetDateTime;

mod mastodon;
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

/// Send a page to PagerDuty
///
/// It does the following
/// 1. Get the environment variables
/// 2. Create a PagerDuty event
/// 3. Send the event
/// 4. Log the result
async fn send_page(status_response: Option<String>) {
    let vars = env::vars().collect::<HashMap<String, String>>();
    let e = Event::AlertTrigger(AlertTrigger {
        payload: AlertTriggerPayload::<String> {
            summary: "Parts of the website are down".to_owned(),
            source: "sachiniyer.com".to_owned(),
            timestamp: Some(OffsetDateTime::now_utc()),
            severity: Severity::Info,
            component: Some("website".to_owned()),
            group: Some("website status".to_owned()),
            class: Some("prod".to_owned()),
            custom_details: status_response,
        },
        dedup_key: Some(OffsetDateTime::now_utc().to_string()),
        images: None,
        links: None,
        client: None,
        client_url: None,
    });

    let result = tokio::task::spawn_blocking(move || {
        EventsV2::new(
            vars.get("PAGER_DUTY_KEY").unwrap().to_string(),
            Some("status".to_owned()),
        )
        .unwrap()
        .event(e)
    })
    .await
    .unwrap();
    println!("Result: {:?}", result);
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
            send_page(Some(status.to_string())).await;
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
        send_page(None).await;
        return Err(Error::from("Error in the lambda function"));
    }
    result
}
