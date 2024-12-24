use pagerduty_rs::eventsv2sync::*;
use pagerduty_rs::types::*;
use std::collections::HashMap;
use std::env;
use time::OffsetDateTime;

/// Send a page to PagerDuty
///
/// It does the following
/// 1. Get the environment variables
/// 2. Create a PagerDuty event
/// 3. Send the event
/// 4. Log the result
pub async fn send_page(status_response: Option<String>) {
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
