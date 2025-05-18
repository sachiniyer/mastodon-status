use chrono::{DateTime, Utc};
use megalodon::megalodon::GetAccountStatusesInputOptions;
use megalodon::{error, generator, Megalodon, SNS};
use std::collections::HashMap;
use std::env;

/// Login to the mastodon instance and verify the credentials
///
/// # Returns
/// - Ok((client, id)) if the login is successful
/// - Err(error) if the login fails
pub async fn login() -> Result<(Box<dyn Megalodon + Send + Sync>, String), error::Error> {
    let vars = env::vars().collect::<HashMap<String, String>>();
    // already checked if these exist in the start of the function calling
    let url = vars.get("INSTANCE_URL").unwrap().clone();
    let token = vars.get("ACCESS_TOKEN").unwrap().clone();
    verify_credentials(url.as_str(), token).await
}

/// Verify the credentials of the user
///
/// # Arguments
/// - url: The mastodon instance url
/// - access_token: The access token of the user
///
/// # Returns
/// - Ok((client, id)) if the login is successful
/// - Err(error) if the login fails
async fn verify_credentials(
    url: &str,
    access_token: String,
) -> Result<(Box<dyn Megalodon + Send + Sync>, String), error::Error> {
    let client = generator(SNS::Mastodon, url.to_string(), Some(access_token), None);
    let res = client.verify_account_credentials().await?;
    let id = res.json().id;
    Ok((client, id))
}

/// Get the latest post of the user
///
/// # Arguments
/// - client: The client object
/// - id: The id of the user
/// - max_id: The max_id of the post
///
/// # Returns
/// - Ok(post) if the post is successfully fetched
/// - Err(error) if the post is not fetched
pub async fn get_post(
    client: &Box<dyn Megalodon + Send + Sync>,
    id: &String,
    max_id: Option<String>,
) -> Result<(String, DateTime<Utc>), error::Error> {
    let res = client
        .get_account_statuses(
            id.to_string(),
            Some(&GetAccountStatusesInputOptions {
                limit: Some(1),
                max_id,
                since_id: None,
                pinned: None,
                exclude_replies: None,
                exclude_reblogs: None,
                only_media: None,
                only_public: None,
            }),
        )
        .await?;
    if res.json().len() == 0 {
        panic!("No post found");
    }
    // This is a little dangerous, but easiest way to get the right post.
    if res.json()[0].content.contains("SACHIN UPDATE") {
        // https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html
        return Box::pin(get_post(client, id, Some(res.json()[0].id.clone()))).await;
    }
    Ok((
        res.json()[0].content.clone(),
        res.json()[0].created_at.clone(),
    ))
}

/// Send a post to the mastodon instance
///
/// # Arguments
/// - client: The client object
/// - status: The status to be posted
///
/// # Returns
/// - Ok(()) if the post is successfully sent
/// - Err(error) if the post is not sent
pub async fn send_post(
    client: &Box<dyn Megalodon + Send + Sync>,
    status: String,
) -> Result<(), error::Error> {
    client.post_status(status, None).await?;
    Ok(())
}
