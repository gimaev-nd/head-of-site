use std::time::Duration;

use reqwest::Client;
use url::Url;

use crate::parse::{AppError, HeadInfo, detect_image_size, extract_head, parse_http_url};

const FETCH_TIMEOUT: Duration = Duration::from_secs(15);

/// Fetches a page and extracts the information the task describes.
pub async fn fetch_and_parse(raw_url: &str) -> Result<HeadInfo, AppError> {
    let url = parse_http_url(raw_url)?;

    let client = Client::builder()
        .timeout(FETCH_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(concat!("head-of-site/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| AppError::FetchFailed(e.to_string()))?;

    let response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|e| AppError::FetchFailed(describe_reqwest_error(&e)))?;

    let html = response
        .text()
        .await
        .map_err(|e| AppError::FetchFailed(format!("failed to read body: {e}")))?;

    let mut info = extract_head(&html, &url);
    fill_favicon_sizes(&mut info, &client).await;

    Ok(info)
}

/// Downloads favicons that lack a `sizes` attribute to determine their size.
async fn fill_favicon_sizes(info: &mut HeadInfo, client: &Client) {
    for favicon in &mut info.favicons {
        if favicon.size.is_some() {
            continue;
        }
        if let Some(size) = fetch_image_size(client, &favicon.url).await {
            favicon.size = Some(size);
        }
    }
}

async fn fetch_image_size(client: &Client, url: &str) -> Option<String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }

    let bytes = client.get(url).send().await.ok()?.bytes().await.ok()?;
    let (width, height) = detect_image_size(&bytes)?;
    Some(format!("{width}x{height}"))
}

fn describe_reqwest_error(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "timeout".to_string()
    } else if error.is_connect() {
        "connection failed".to_string()
    } else if error.is_redirect() {
        "too many redirects".to_string()
    } else {
        "request failed".to_string()
    }
}

/// Exposed for tests.
pub fn _parse_url_for_test(raw: &str) -> Result<Url, AppError> {
    parse_http_url(raw)
}
