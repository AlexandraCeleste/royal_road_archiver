use chrono::prelude::{DateTime, Utc};
use reqwest::blocking::Client;
use crate::{http_get, html_query};

#[derive(Clone)]
pub struct Book {
    pub reqwest_client: Client,

    pub title: String,
    pub author: String,
    pub description: String, // <- Not used  due to description causing some epub readers to crash and burn.
    pub cover_img_url: String,
    pub date_archived: DateTime<Utc>,
    pub chapter_urls: Vec<String>
}

impl Book {
    pub fn new(url: &str) -> Book {
        let client = Client::new();
        let htmldoc = http_get::get_html_blocking(url, &client);

        return Book { 
            reqwest_client: client,
            title: html_query::get_title(&htmldoc), 
            author: html_query::get_author(&htmldoc),
            description: html_query::get_description(&htmldoc),
            cover_img_url: html_query::get_cover_img_url(&htmldoc), 
            date_archived: Utc::now(), 
            chapter_urls: html_query::get_chapter_urls(&htmldoc) }
    }
}