mod book;
mod generate_epub;
mod generate_md;
mod html_query;
mod http_get;

use crate::book::Book;

fn main() {
    let url = "https://www.royalroad.com/fiction/39098/menschenjaeger";
    let book:Book = Book::new(url);
    
    match 2 {
        1 => generate_md::generate(book),
        2 => generate_epub::generate(book),
        _ => ()
    }
}