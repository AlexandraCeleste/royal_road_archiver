use crate::{book::Book, http_get, html_query};

use std::fs::File;
use std::io::Write;
use chrono::Datelike;
use html2md::parse_html;

pub fn generate(book: Book) {
    let path = [&book.title, ".md"].join("");
    let mut output = File::create(path).unwrap();

    let buf = [
            book.title, 
            "by:\n  ~".to_string(),
            book.author].join(" ");
    output.write(buf.as_bytes()).unwrap(); // Write the title and author to the .md file.

    let buf = [
        book.date_archived.day().to_string(),
        book.date_archived.month().to_string(),
        book.date_archived.year().to_string()].join(":");
    output.write("\n\nArchived on: ".as_bytes()).unwrap();
    output.write(buf.as_bytes()).unwrap();
    output.write("\n\n".as_bytes()).unwrap(); // Write the date archived on to the .md file.

    for url in &book.chapter_urls {
        let htmldoc = http_get::get_html_blocking(&url, &book.reqwest_client);

        let buf = ["**", &html_query::get_chapter_name(&htmldoc), "**"].join("");
        output.write(buf.as_bytes()).unwrap();
        output.write("\n".as_bytes()).unwrap();
        output.write(parse_html(&html_query::get_chapter_content(&htmldoc)).as_bytes()).unwrap();
        output.write("\n\n".as_bytes()).unwrap();
    }
}