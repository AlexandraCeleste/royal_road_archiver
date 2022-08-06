#![allow(non_snake_case)] // Disable the snake case warning.
#![allow(dead_code)] // Disable warnings for unused code.
#![allow(unused_imports)] // Disable warnings for unused imports.

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

use std::io;
use std::fs::File;
use std::io::Write;


struct Book { // The book struct includes information about the book ready to be bundled into the Epub.
    Title: String,
    Author: String,
    CoverArt: String,
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
            CoverArt: imageUrl,
            DateCreated: Utc::now(),
            Chapters: get_chapters(&document), 
            BookUrl: url.to_owned(),
        };
    }

    fn populate_chapter_content(&mut self) {
        for chapter in self.Chapters.iter_mut() {
            chapter.get_chapter_content();
        }
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

    make_epub();
}

fn make_epub() {
    let builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
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