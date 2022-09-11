use reqwest::blocking::Client;
use scraper::Html;

pub fn get_html_blocking(url: &str, client: &Client) -> Html{
    let body = client.get(url).send().expect("Unable to connect");

    return Html::parse_document(&body.text().expect("Error getting html content"));
}

pub fn get_img_blocking(url: &str, client: &Client) -> Vec<u8> { // Download and return the requested image as a vector of bytes.
    let mut body = client.get(url).send().expect("Unable to connect");
    
    let mut buf: Vec<u8> = vec![];
    body.copy_to(&mut buf).expect("Unable to get image bytes");

    return buf;
}