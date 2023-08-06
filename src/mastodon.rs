use megalodon::megalodon::GetAccountStatusesInputOptions;
use megalodon::{error, generator, Megalodon, SNS};
use std::collections::HashMap;
use std::env;

pub async fn login() -> Result<(Box<dyn Megalodon + Send + Sync>, String), error::Error> {
    let vars = env::vars().collect::<HashMap<String, String>>();
    let url = vars.get("INSTANCE_URL").unwrap().clone();
    let token = vars.get("ACCESS_TOKEN").unwrap().clone();
    verify_credentials(url.as_str(), token).await
}

async fn verify_credentials(
    url: &str,
    access_token: String,
) -> Result<(Box<dyn Megalodon + Send + Sync>, String), error::Error> {
    let client = generator(SNS::Mastodon, url.to_string(), Some(access_token), None);
    let res = client.verify_account_credentials().await?;
    let id = res.json().id;
    Ok((client, id))
}

pub async fn get_post(
    client: &Box<dyn Megalodon + Send + Sync>,
    id: String,
) -> Result<String, error::Error> {
    let res = client
        .get_account_statuses(
            id,
            Some(&GetAccountStatusesInputOptions {
                limit: Some(1),
                max_id: None,
                since_id: None,
                pinned: None,
                exclude_replies: None,
                exclude_reblogs: None,
                only_media: None,
            }),
        )
        .await;
    let content = res.unwrap().json();
    let content = content[0].content.clone();
    Ok(content)
}

pub async fn send_post(
    client: &Box<dyn Megalodon + Send + Sync>,
    status: String,
) -> Result<(), error::Error> {
    client.post_status(status, None).await?;
    Ok(())
}
