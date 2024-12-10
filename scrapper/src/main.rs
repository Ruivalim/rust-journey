use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://dev.to/ruivalim/";

    let mut headers = HeaderMap::new();

    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
    ));

    headers.insert(
        "accept",
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        ),
    );

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?;

    let body = client.get(url).send()?.text()?;

    let document = Html::parse_document(&body);

    let selector = Selector::parse(".crayons-story__title a").unwrap();

    for element in document.select(&selector) {
        let title = element.text().collect::<Vec<_>>().join("");
        let link = element.value().attr("href").unwrap_or("No link");
        println!("{} - {}", title, link);
    }

    Ok(())
}
