mod book;
mod extensions;
mod generate_epub;
mod generate_md;
mod html_query;
mod http_get;
use std::env;
use std::process;

use crate::book::Book;
use url::{Url};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"-help".to_string()) { print_help(); }
    if args.contains(&"--help".to_string()) { print_help(); }

    validate_args(&args);

    let book:Book = Book::new(&args[1]);
    
    let mut noimages = false;
    if args.contains(&"--noimages".to_string()) { noimages = true; }

    let mut ran = false;
    if args.contains(&"--markdown".to_string()) { generate_md::generate(&book); ran = true;}
    if args.contains(&"--epub".to_string()) { generate_epub::generate(&book, noimages); ran = true;}

    if !ran { generate_epub::generate(&book, noimages); } // If no options were selected the default runs (Currently epub).
}

fn validate_args(args: &Vec<String>) {
    if args.len() == 1 { print_help(); }

    let url = Url::parse(&args[1]);
    let url = match url {
        Ok(url) => url,
        Err(_) => {println!("Error: Please enter a valid URL"); process::exit(0); },
    };
    if url.host_str().unwrap() != "www.royalroad.com" { panic!("URL must point to www.royalroad.com"); }

    for i in 2..args.len() {
        match args[i].as_str() {
            "--noimages" => (),
            "--markdown" => (),
            "--epub" => (),
            _ => { println!("Error: {} is not a valid argument. do: royal_road_archiver --help", args[i]); process::exit(0); }
        }
    }
}

fn print_help() {
    println!("Royal Road Archiver help:");
    println!("Standard usage: royal_road_archiver <royal road book url>");
    println!("E.G:\nroyal_road_archiver https://www.royalroad.com/fiction/26675/a-journey-of-black-and-red");
    println!("");
    println!("Optional arguments:\n");
    println!("  --noimages");
    println!("  Removes all images from the book. Usefull for making smaller epubs\n");
    println!("\nSupported output types:");
    println!("  --markdown");
    println!("  --epub  <Default>");
    
    process::exit(0);
}