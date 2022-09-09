use crate::book::Book;

use epub_builder::{EpubBuilder, ZipLibrary, EpubContent, ReferenceType, Result};

const HEAD: &str = r#"<html><body>"#;
const TAIL: &str = r#"</body></html>"#;

pub fn generate(book: Book) {
    let mut epub_build = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();

    add_metadata(&book, &mut epub_build);
}

pub fn add_metadata(book:&Book, epub_build: &mut EpubBuilder<ZipLibrary>) {
    epub_build.metadata("author", book.author.clone()).expect("Unable to add author metadata");
    epub_build.metadata("title", book.title.clone()).expect("Unable to add title metadata");
    epub_build.metadata("description", book.description.clone()).expect("Unable to add description metadata");
    epub_build.metadata("lang", "en").expect("Unable to add lang -en metadata");
}