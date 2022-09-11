use crate::html_query;
use crate::{book::Book, http_get};
use epub_builder::{EpubBuilder, ZipLibrary, EpubContent, ReferenceType};
use chrono::Datelike;
use std::fs::File;
use std::io::Write;

use scraper::{Html, Selector};

const HEAD: &str = r#"<html><body>"#;
const TAIL: &str = r#"</body></html>"#;

pub fn generate(book: Book) {
    let mut epub_build = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();

    add_metadata(&book, &mut epub_build);
    add_cover(&book, &mut epub_build);
    epub_build.inline_toc(); // Add the Table of Contents after the cover page.
    add_chapters(&book, &mut epub_build);

    let mut epub: Vec<u8> = vec![];
    epub_build.generate(&mut epub).expect("Unable to generate epub data");

    let mut output = File::create(format!("{}.epub", book.title)).expect("Unable to create epub file");
    output.write_all(epub.as_slice()).expect("Unable to write epub data to epub file");
}

pub fn add_metadata(book: &Book, epub_build: &mut EpubBuilder<ZipLibrary>) {
    epub_build.metadata("author", book.author.clone()).expect("Unable to add author metadata");
    epub_build.metadata("title", book.title.clone()).expect("Unable to add title metadata");
    //epub_build.metadata("description", book.description.clone()).expect("Unable to add description metadata"); <- For some reason this causes some epub readers to crash and burn.
    epub_build.metadata("lang", "en").expect("Unable to add lang -en metadata");
}

pub fn add_cover(book: &Book, epub_build: &mut EpubBuilder<ZipLibrary>) {
    let cover = http_get::get_img_blocking(&book.cover_img_url, &book.reqwest_client);
    epub_build.add_cover_image("cover.jpg", cover.as_slice(), "image/jpg").expect("Unable to add cover image.");
    // ^ as_slice needed due to vec<u8> not supporting std::io::read.

    let body = format!(r#"<div style="text-align: center;"><h1>{}</h1>
        <img src="{}"/>
        <h2>by: {}</h2>
        <h3>Archived on: {}:{}:{}</h3></div>"#, 
        book.title,
        "cover.jpg",
        book.author,
        book.date_archived.day(), book.date_archived.month(), book.date_archived.year());
    
    let xhtml = format!("{}{}{}", HEAD, body, TAIL);

    epub_build.add_content(EpubContent::new("title.xhtml", xhtml.as_bytes())
    .title("Cover")
    .reftype(ReferenceType::Cover)).expect("Unable to add cover");
}

pub fn add_chapters(book: &Book, epub_build: &mut EpubBuilder<ZipLibrary>) {

    for url in &book.chapter_urls {
        let htmldoc = http_get::get_html_blocking(&url, &book.reqwest_client);
        let chapter_title = html_query::get_chapter_name(&htmldoc).replace("&nbsp;", " "); // Remeber to remove &nbsp;
        let mut chapter_content = html_query::get_chapter_content(&htmldoc);

        chapter_content = chapter_content.replace("<br>", "<br/>"); // Remeber to close the br tags or epub readers WILL kill you.
        chapter_content = chapter_content.replace("&nbsp;", " "); // Remeber to remove &nbsp;

        let xhtml = format!("{}{}{}", HEAD, chapter_content, TAIL);

        epub_build.add_content(EpubContent::new(format!("{}.xhtml", chapter_title), xhtml.as_bytes())
            .title(chapter_title)
            .reftype(ReferenceType::Text)).expect("Unable to add cover");
    }
}