use std::path::Path;

use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;

pub async fn generate_images(
    base_dir: &Path,
    urls: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    // create a `Browser` that spawns a `chromium` process running with UI (`with_head()`, headless is default)
    // and the handler that drives the websocket etc.
    let (mut browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    // spawn a new task that continuously polls the handler
    let handle = tokio::task::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    // create a new browser page and navigate to the url
    let page = browser.new_page("https://en.wikipedia.org").await?;

    Ok(())
}
