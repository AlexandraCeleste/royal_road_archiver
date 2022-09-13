use crate::extensions::VecStringExt;
use crate::html_query;
use crate::{book::Book, http_get};
use epub_builder::{EpubBuilder, ZipLibrary, EpubContent, ReferenceType};
use chrono::Datelike;
use scraper::Html;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

const HEAD: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">"#;
const TAIL: &str = r#"</html>"#;
const CSS: &str = "";

pub fn generate(book: &Book, noimages: bool) {
    let mut epub_build = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();

    epub_build.stylesheet(CSS.as_bytes()).unwrap();

    add_metadata(&book, &mut epub_build);
    add_cover(&book, &mut epub_build);
    epub_build.inline_toc(); // Add the Table of Contents after the cover page.
    add_chapters(&book, &mut epub_build, noimages);

    let mut epub: Vec<u8> = vec![];
    epub_build.generate(&mut epub).expect("Unable to generate epub data");

    let mut output = File::create(format!("{}.epub", book.title)).expect("Unable to create epub file");
    output.write_all(epub.as_slice()).expect("Unable to write epub data to epub file");
}

fn add_metadata(book: &Book, epub_build: &mut EpubBuilder<ZipLibrary>) {
    epub_build.metadata("author", book.author.clone()).expect("Unable to add author metadata");
    epub_build.metadata("title", book.title.clone()).expect("Unable to add title metadata");
    //epub_build.metadata("description", book.description.clone()).expect("Unable to add description metadata"); <- For some reason this causes some epub readers to crash and burn.
    epub_build.metadata("lang", "en").expect("Unable to add lang -en metadata");
}

fn add_cover(book: &Book, epub_build: &mut EpubBuilder<ZipLibrary>) {
    let cover = http_get::get_img_blocking(&book.cover_img_url, &book.reqwest_client);
    epub_build.add_cover_image("cover.jpg", cover.as_slice(), "image/jpg").expect("Unable to add cover image.");
    // ^ as_slice needed due to vec<u8> not supporting std::io::read.

    let body = format!(r#"<head></head><body><div style="text-align: center;"><h1>{}</h1>
        <img src="{}"/>
        <h2>by: {}</h2>
        <h3>Archived on: {}:{}:{}</h3></div></body>"#, 
        book.title,
        "cover.jpg",
        book.author,
        book.date_archived.day(), book.date_archived.month(), book.date_archived.year());
    
    let xhtml = format!("{}{}{}", HEAD, body, TAIL);

    epub_build.add_content(EpubContent::new("title.xhtml", xhtml.as_bytes())
    .title("Cover")
    .reftype(ReferenceType::Cover)).expect("Unable to add cover");
}

fn add_chapters(book: &Book, epub_build: &mut EpubBuilder<ZipLibrary>, noimages: bool) {
    let mut chapter_n: u32 = 0;
    let mut used_srcs: HashMap<String, String> = HashMap::new();

    for url in &book.chapter_urls {
        let htmldoc = http_get::get_html_blocking(&url, &book.reqwest_client);
        let chapter_title = html_query::get_chapter_name(&htmldoc).replace("&nbsp;", " "); // Remeber to remove &nbsp; It causes some epub readers to crash.
        let mut chapter_content = html_query::get_chapter_content(&htmldoc);
    
        match noimages {
            true => chapter_content = remove_img_tags(Html::parse_fragment(&chapter_content)),
            false => chapter_content = add_images(Html::parse_fragment(&chapter_content), &chapter_n, &book, epub_build, &mut used_srcs)
        }
        chapter_content = format!("<head><title>{}</title></head><body><h1>{}</h1>{}</body>", chapter_title, chapter_title, chapter_content);
        chapter_content = chapter_content.replace("<br>", "<br/>"); // Remeber to close the br tags or epub readers WILL kill you.
        chapter_content = chapter_content.replace("&nbsp;", " "); // Remeber to remove &nbsp;

        let xhtml = format!("{}{}{}", HEAD, chapter_content, TAIL); // To whom it may concern, XHTML is a piece of garbage and if you had anything to do in designing the spec I will fucking kneecap you. Yours sincerely ~ Raine

        epub_build.add_content(EpubContent::new(format!("chapter_{}.xhtml", chapter_n), xhtml.as_bytes())
            .title(chapter_title)
            .reftype(ReferenceType::Text)).expect("Unable to add cover");
        
        chapter_n +=1
    }
}

fn add_images(fragment: Html, chapter_n: &u32, book: &Book, epub_build: &mut EpubBuilder<ZipLibrary>, used_srcs: &mut HashMap<String, String>) -> String {
    let img_tags = html_query::get_img_tags(&fragment);
    let mut content = fragment.root_element().html();

    for i in 0..img_tags.0.len() {
        let src: String;
        let mut width = String::new();
        let mut height = String::new();

        match &img_tags.1[i] { // If no img src is present the tag will be removed from the content and the iteration skipped.
            None => { content = content.replacen(&img_tags.0[i], "", 1); continue; },
            Some(x) => {
                if !used_srcs.contains_key(x) {

                    let img = &http_get::get_img_blocking(&x, &book.reqwest_client);
                    let path: String;
                    let mime_type: String;

                    match image::guess_format(&img) {
                        Ok(format) => {
                            match format {
                                image::ImageFormat::Jpeg => {path = format!("id{}_{}.jpg", chapter_n, i); mime_type = "image/jpeg".to_string(); },
                                image::ImageFormat::Png => {path = format!("id{}_{}.png", chapter_n, i); mime_type = "image/png".to_string(); },
                                _ => { // If format isn't supported the image tag is removed and code skips to the next.
                                    println!("Image format: {:?} is not supported. Ask a developer to support it.", format);
                                    content = content.replacen(&img_tags.0[i], "", 1);
                                    continue;
                                }
                            }
                        },
                        Err(_) => { content = content.replacen(&img_tags.0[i], "", 1); continue; } // If format can't be determined, remove the tag and skip to the next.
                    }
                    used_srcs.insert(x.clone(), path.clone());
                    epub_build.add_resource(path, img.as_slice(), mime_type).unwrap();

                }
                src = format!(r#"src="{}""#, used_srcs.get(x).unwrap());
            }
        }
        match img_tags.2[i] {
            None => (),
            Some(i) => width = format!(r#"width="{}""#, i)
        }
        match img_tags.3[i] {
            None => (),
            Some(i) => height = format!(r#"height="{}""#, i)
        }

        let tag = format!("<img {} {} {}/>", src, width, height);
        content = content.replacen(&img_tags.0[i], &tag, 1);
    }

    return content;
}

fn remove_img_tags(fragment: Html) -> String {
    let img_tags = html_query::get_img_tags(&fragment);
    let img_tags = img_tags.0.remove_duplicates();

    let mut content = fragment.root_element().html();

    for tag in &img_tags {
        content = content.replace(tag, "")
    }
    return content;
}