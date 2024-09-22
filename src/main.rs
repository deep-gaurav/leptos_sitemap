use clap::Parser;
use std::path::{Path, PathBuf};
use std::fs;
use tokio::fs as tokio_fs;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Cursor;
use async_recursion::async_recursion;

#[derive(Parser)]
struct Cli {
    /// The root directory to start the search
    #[arg(short, long)]
    dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Collect all index.html files recursively
    let mut urls = Vec::new();
    find_index_html(&args.dir, &args.dir, &mut urls).await?;

    // Generate the sitemap.xml content
    let sitemap = generate_sitemap(&urls)?;
    tokio_fs::write("sitemap.xml", sitemap).await?;

    println!("Sitemap generated: sitemap.xml");

    Ok(())
}

#[async_recursion]
async fn find_index_html(base_dir: &Path, dir: &Path, urls: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = tokio_fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            find_index_html(base_dir,&path, urls).await?;
        } else if path.is_file() && path.file_name() == Some("index.html".as_ref()) {
            // If we found an index.html, add the full path as a URL
            let path = path.strip_prefix(base_dir);
            if let Ok(path) = path {
                if let Some(path_str) = path.to_str() {
                    if let Some(path) = path_str.strip_suffix("index.html"){
                        urls.push(path.to_string());
                    }else{
                        urls.push(path_str.to_string());
                    }
                }
            }
        }
    }

    Ok(())
}

fn generate_sitemap(urls: &[String]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Write the XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // Start <urlset> tag
    let mut urlset = BytesStart::new("urlset");
    urlset.push_attribute(("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9"));
    writer.write_event(Event::Start(urlset))?;

    // Add each <url> entry
    for url in urls {
        let mut url_tag = BytesStart::new("url");
        writer.write_event(Event::Start(url_tag))?;

        let mut loc_tag = BytesStart::new("loc");
        writer.write_event(Event::Start(loc_tag))?;
        writer.write_event(Event::Text(BytesText::new(url.as_str())))?;
        writer.write_event(Event::End(BytesEnd::new("loc")))?;

        writer.write_event(Event::End(BytesEnd::new("url")))?;
    }

    // End <urlset> tag
    writer.write_event(Event::End(BytesEnd::new("urlset")))?;

    Ok(writer.into_inner().into_inner())
}
