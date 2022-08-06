#![allow(non_snake_case)] // Disable the snake case warning.

//use std::env;
use scraper::{Html, Selector};
use reqwest::blocking;

struct Chapter {
    Name: String,
    Url: String,
    Content: String,
}

fn main() {
    //let args: Vec<String> = env::args().collect();
    //let Url = args[1].to_owned();

    let Url = String::from("https://www.royalroad.com/fiction/28023/katalepsis");

    let mut chapters = get_chapters(Url);

    for i in 0..chapters.len() {
        chapters[i].Content = get_chapter_content(&chapters[i].Url);
        println!();
    }
}

fn get_htmldocument(Url: &String) -> Html {
    let body = blocking::get(Url)
        .expect("The get request did a fucky whucky.");
    let rawhtml = &body.text().expect("Error parsing raw html.");

    return Html::parse_document(&rawhtml);
}

fn get_chapter_content(Url: &String) -> String {
    let document = get_htmldocument(&Url);

    let selector = Selector::parse("div").unwrap();

    for content in document.select(&selector) {
        match content.value().classes().into_iter().find(|&x| x == "chapter-content" ) { // match to find only the div that contains the class "chapter-content".
            None => continue,
            Some(_) => { return content.inner_html(); }
        }
    }
    panic!("Couldn't find chaper-content");
}

fn get_chapters(Url: String) -> Vec<Chapter> {

    let mut chapters: Vec<Chapter> = Vec::new();
    let document = get_htmldocument(&Url);
    
    let selector = Selector::parse("tbody").unwrap(); // Find <tbody><\tbody> in the html doc.

    if document.select(&selector).count() != 1 { panic!("Error: Royal Road has changed their html scheme!") }
    for element in document.select(&selector) { // Unless royal road change their html scheme, their should only be 1 tbody element.
        let htmlfragment = Html::parse_fragment(&element.html()); // Make a new html fragment that contains only <tbody><\tbody> and it's children.
        let selector = Selector::parse("a, href").unwrap(); // Find just the chapter names and links from their a href <> tags.

        let mut count:u32 = 0;
        for link in htmlfragment.select(&selector) {
            if count % 2 as u32 != 0 { count+=1; continue; } // If count is not even skip over iteration.
            count +=1;

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