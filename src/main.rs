#![allow(non_snake_case)] // Disable the snake case warning.
#![allow(dead_code)] // Disable warnings for unused code.
#![allow(unused_imports)] // Disable warnings for unused imports.

use image::DynamicImage;
//use std::env;
use scraper::{Html, Selector};
use reqwest::blocking;

use epub_builder::EpubBuilder;
use epub_builder::Result;
use epub_builder::ZipLibrary;
use epub_builder::EpubContent;
use epub_builder::ReferenceType;
use epub_builder::TocElement;

use chrono::prelude::*;

use image;

use std::collections::VecDeque;

use std::fs;
use std::io;
use std::fs::{File, create_dir};
use std::io::Read;
use std::io::Write;


struct Book { // The book struct includes information about the book ready to be bundled into the Epub.
    Title: String,
    Author: String,
    CoverArtUrl: String,
    DateCreated: DateTime<Utc>,
    Chapters: Vec<Chapter>,
    BookUrl: String,
}

impl Book {
    fn new(document: &Html, url: String ) -> Book {
        let mut title = String::new();
        let mut author = String::new();
        let mut imageUrl = String::new();

        let selector = Selector::parse("h1").unwrap();
        for element in document.select(&selector) {
            match element.value().attr("property") {
                None => continue,
                Some(x) => {
                    if x == "name" { title = element.inner_html(); break;}
                }
            }
        }

        let selector = Selector::parse("span").unwrap();
        for element in document.select(&selector) {
            match element.value().attr("property") {
                None => continue,
                Some(x) => { 
                    if x == "name" {
                        let fragment = Html::parse_fragment(element.inner_html().as_str());
                        let selector = Selector::parse("a").unwrap();
                        for a in fragment.select(&selector) {
                            author = a.inner_html(); break;
                        }
                    }
                }
            }
        }

        let selector = Selector::parse("img").unwrap();
        for element in document.select(&selector) {
            match element.value().attr("alt") {
                None => continue,
                Some(x) => {
                    if x == title { 
                        imageUrl = element.value().attr("src").unwrap().to_owned();
                        break; 
                    }
                }
            }
        }

        return Book { 
            Title: title,
            Author: author,
            CoverArtUrl: imageUrl,
            DateCreated: Utc::now(),
            Chapters: get_chapters(&document), 
            BookUrl: url.to_owned(),
        };
    }

    fn populate_chapter_content(&mut self) {
        for chapter in self.Chapters.iter_mut() {
            chapter.get_chapter_content();
            break; // Remove this to get all chapter content !!!
        }
    }

    fn get_image(&self) -> DynamicImage {

        let img_bytes = blocking::get(&self.CoverArtUrl).expect("Error getting image")
        .bytes().expect("Error parsing image bytes."); // Download image bytes from the url with a blocking get request.

        let image = image::load_from_memory(&img_bytes).expect("Image error"); // load image from memory.
        
        return image;
    }
}

struct Chapter { // The chapter struct holds the chapters name, Url, and raw content.
    Name: String,
    Url: String,
    Content: String,
}

impl Chapter {
    fn get_chapter_content(&mut self) {
        let document = get_htmldocument(&self.Url);
    
        let selector = Selector::parse("div").unwrap();
    
        for content in document.select(&selector) {
            match content.value().classes().into_iter().find(|&x| x == "chapter-content" ) { // match to find only the div that contains the class "chapter-content".
                None => continue,
                Some(_) => { self.Content = content.inner_html(); break; }
            }
        }
    }
}

fn main() {
    //let args: Vec<String> = env::args().collect();
    //let Url = args[1].to_owned();

    let Url = String::from("https://www.royalroad.com/fiction/28023/katalepsis");

    let mut book:Book = Book::new(&get_htmldocument(&Url), Url);
    book.populate_chapter_content();

    let epub = make_epub(&book);

    let mut output = File::create("test.epub").expect("Unable to create 'file.epub'");

    output.write(&epub).expect("Unable to write epub to 'file.epub'");
}

fn make_epub(book:&Book) -> Vec<u8> {
    let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();

    builder.metadata("title", &book.Title).expect("Error setting title.");
    builder.metadata("author", &book.Author).expect("Error setting author.");
    builder.metadata("lang", "en").expect("Error setting lang to en.");

    builder.add_cover_image("cover.jpg", book.get_image().as_bytes() ,"image/jpg").expect("Unable to add cover image.");

    let xml = generate_xml(&book.Chapters[0].Content);

    builder.add_content(EpubContent::new("chapter_1.xhtml", xml.as_bytes())
        .title("Chapter 1")
        .reftype(ReferenceType::Text)).unwrap();

    let mut epub: Vec<u8> = Vec::new();
    builder.generate(&mut epub).unwrap();

    return epub;
}

fn fix_xml(xml: &String) -> String {
    
    return String::new();
}

fn generate_xml(content: &String) -> String {
    let head = r#"<?xml version ="1.0" encoding ="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body>"#;

    let tail = r#"</body></xml>"#;

    let mut output = String::new();

    let fixed = fix_xml(content);

    output.push_str(&head);
    output.push_str(&fixed);
    output.push_str(&tail);

    return output;
}

fn get_htmldocument(Url: &String) -> Html {
    let body = blocking::get(Url)
        .expect("The get request did a fucky whucky.");
    let rawhtml = &body.text().expect("Error parsing raw html.");

    return Html::parse_document(&rawhtml);
}

fn get_chapters(document: &Html) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = Vec::new();
    
    let selector = Selector::parse("tbody").unwrap(); // Find <tbody><\tbody> in the html doc.

    if document.select(&selector).count() != 1 { panic!("Error: Royal Road has changed their html scheme!") }
    for element in document.select(&selector) { // Unless royal road change their html scheme, their should only be 1 tbody element.
        let htmlfragment = Html::parse_fragment(&element.html()); // Make a new html fragment that contains only <tbody><\tbody> and it's children.
        let selector = Selector::parse("a, href").unwrap(); // Find just the chapter names and links from their a href <> tags.

        for link in htmlfragment.select(&selector).step_by(2) { // Step 2 in order to ignore repeating a href elements.
  
            let chapter = Chapter { 
                Name: get_chapter_name(link.inner_html()).replace("&nbsp;", " "), 
                Url: [String::from("https://www.royalroad.com") , link.value().attr("href").unwrap().to_owned()].join(""),
                Content: String::new(),
            };
            chapters.push(chapter);
        }
    }
    return chapters;
}

fn get_chapter_name(input: String) -> String { // Function to remove random whitespace around text inside the <a href>...<\a> tags.
    let mut output: Vec<char> = Vec::new();

    let mut is_name = false;
    for char in input.chars() {
        if char.is_alphanumeric() { is_name = true; }
        else if char == '\n' { is_name = false; }

        if is_name { output.push(char); }
    }
    return output.into_iter().collect();
}