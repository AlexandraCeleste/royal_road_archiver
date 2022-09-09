use scraper::{Html, Selector};

pub fn get_title(htmldoc: &Html) -> String {

    let selector = Selector::parse("h1").unwrap();
    for element in htmldoc.select(&selector) {
        match element.value().attr("property") {
            None => continue,
            Some(x) => {
                if x == "name" { return element.inner_html();}
            }
        }
    }
    panic!("Unable to book name.");
}

pub fn get_author(htmldoc: &Html) -> String {

    let selector = Selector::parse("span").unwrap();
    for element in htmldoc.select(&selector) {
        match element.value().attr("property") {
            None => continue,
            Some(x) => {
                if x == "name" {
                    let selector = Selector::parse("a").unwrap();
                    let element = element.select(&selector).next().unwrap();
                    return element.inner_html();
                }
            }
        }
    }
    panic!("Unable to find book author.");
}

pub fn get_description(htmldoc: &Html) -> String {
    let selector = Selector::parse("div").unwrap();
    for element in htmldoc.select(&selector) {
        match element.value().attr("property") {
            None => continue,
            Some(x) => {
                if x == "description" {
                    let fragment = Html::parse_fragment(&element.inner_html());
                    
                    let mut description = String::new();

                    let selector = Selector::parse("p").unwrap();
                    for element in fragment.select(&selector) {
                        description.push_str(&format!("{}\n", element.inner_html()));
                    }

                    return description;
                }
            }
        }
    }
    panic!("Unable to find description.");
}

pub fn get_cover_img_url(htmldoc: &Html) -> String {
    let selector = Selector::parse("img").unwrap();
    for element in htmldoc.select(&selector) {
        match element.value().attr("class") {
            None => continue,
            Some(x) => {
                if x == "thumbnail inline-block" {
                    return element.value().attr("src").unwrap().to_string();
                }
            }
        }
    }
    panic!("Unable to get cover art url");
}

pub fn get_chapter_urls(htmldoc: &Html) -> Vec<String> {

    let mut chapter_urls: Vec<String> = Vec::new();
    let mut urls: Vec<String> = Vec::new();

    let selector = Selector::parse("tr").unwrap();
    for element in htmldoc.select(&selector) {
        match element.value().attr("class") {
            None => continue,
            Some(x) => {
                if x == "chapter-row" {
                    let selector = Selector::parse("td").unwrap();
                    let element = element.select(&selector).next().unwrap();

                    let selector = Selector::parse("a").unwrap();
                    let element = element.select(&selector).next().unwrap();
                    urls.push(["https://www.royalroad.com",element.value().attr("href").unwrap()].join(""));
                }
            }
        }
    }

    for url in urls {
        chapter_urls.push(url);
    }

    return chapter_urls;
}

pub fn get_chapter_name(htmldoc: &Html) -> String {
    
    let selector = Selector::parse("div").unwrap();
    for element in htmldoc.select(&selector) {
        match element.value().attr("class") {
            None => continue,
            Some(x) => {
                if x == "col-md-5 col-lg-6 col-md-offset-1 text-center md-text-left" {
                    let fragment = Html::parse_fragment(&element.html());
                    let selector = Selector::parse("h1").unwrap();

                    return fragment.select(&selector).next().unwrap().inner_html();
                }
            }
        }
    }
    panic!("Unable to find chapter name.")
}

pub fn get_chapter_content (htmldoc: &Html) -> String {
    
    let selector = Selector::parse("div").unwrap();
    for element in htmldoc.select(&selector) {
        match element.value().attr("class") {
            None => continue,
            Some(x) => {
                if x == "chapter-inner chapter-content" {
                    return element.inner_html();
                }
            }
        }
    }
    panic!("Unable to get chapter content");
}