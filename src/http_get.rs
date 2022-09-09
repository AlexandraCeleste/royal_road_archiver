use reqwest::blocking::Client;
use scraper::Html;

pub fn get_html_blocking(url: &str, client: &Client) -> Html{
    let body = client.get(url).send().expect("Unable to connect");

    return Html::parse_document(&body.text().expect("Error getting html content"));
}