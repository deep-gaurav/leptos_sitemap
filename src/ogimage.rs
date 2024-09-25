use std::path::Path;

use axum::{routing::get_service, Router};
use chromiumoxide::{handler::viewport::Viewport, Browser, BrowserConfig};
use futures::StreamExt;
use tower_http::services::ServeDir;

pub async fn generate_images(
    base_dir: &Path,
    urls: &[String],
    host: &str,
) -> Result<(), Box<dyn std::error::Error>> {



    let static_service = get_service(ServeDir::new(base_dir));

    let app = Router::new().nest_service("/", static_service);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();

    let port = listener.local_addr()?.port();
    let jc =  tokio::spawn(async move {
        axum::serve(listener, app).await
    });


    // Wait for server to start
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // create a `Browser` that spawns a `chromium` process running with UI (`with_head()`, headless is default)
    // and the handler that drives the websocket etc.
    let (mut browser, mut handler) =
        Browser::launch(BrowserConfig::builder().viewport(Some(Viewport{
            width: 1200,
            height: 800,
            ..Default::default()
        })).build()?).await?;
    // spawn a new task that continuously polls the handler
    let handle = tokio::task::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });
    
    // create a new browser page and navigate to the url
    for url in urls.iter() {
        println!("check for url {url}");
        let page = browser.new_page(format!("http://127.0.0.1:{port}/{}",url)).await?;

        let og_div = page.find_element("#og-image").await;
        let og_img = page.find_element(r#"meta[property="og:image"]"#).await;
        if let (Ok(og_div), Ok(og_img)) = (og_div, og_img) {
            let image_name = og_img.attribute("content").await;
            if let Ok(Some(img_name)) = image_name{
                let img_name = img_name.strip_prefix(host).unwrap_or(&img_name);
                let img_name = img_name.replace("/", std::path::MAIN_SEPARATOR_STR);
                println!("img_name {img_name}");
                let path = base_dir.join(Path::new(&img_name));
                og_div.save_screenshot(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Jpeg, &path).await;
                println!("OG Image Generated {path:?}");
            }
        }
    }

    jc.abort();
    Ok(())
}
