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

        let vars = env::vars().collect::<HashMap<String, String>>();
        let ev2 = EventsV2::new(
            vars.get("PAGERDUTY_TOKEN").unwrap().to_string(),
            Some("status".to_owned()),
        )
        .unwrap();
        let e = Event::AlertTrigger(AlertTrigger {
            payload: AlertTriggerPayload::<Option<String>> {
                summary: "Syncronously Test Alert 1 Maximum fields".to_owned(),
                source: "hostname".to_owned(),
                timestamp: Some(OffsetDateTime::now_utc()),
                severity: Severity::Info,
                component: Some("postgres".to_owned()),
                group: Some("prod-datapipe".to_owned()),
                class: Some("deploy".to_owned()),
                custom_details: None,
            },
            dedup_key: Some("something".to_string()),
            images: Some(vec![Image {
                src: "https://polyverse.com/static/img/SplashPageIMG/polyverse_blue.png".to_owned(),
                href: Some("https://polyverse.com".to_owned()),
                alt: Some("The Polyverse Logo".to_owned()),
            }]),
            links: Some(vec![Link {
                href: "https://polyverse.com".to_owned(),
                text: Some("Polyverse homepage".to_owned()),
            }]),
            client: Some("Zerotect".to_owned()),
            client_url: Some("https://github.com/polyverse/zerotect".to_owned()),
        });

        let result = ev2.event(e);
        println!("Result: {:?}", result);
        return Err(Error::from("Error in the lambda function"));
    }
    result
}
