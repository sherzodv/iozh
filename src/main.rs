use std::path::PathBuf;

extern crate pest;
#[macro_use]
extern crate pest_derive;


mod ast;
mod parser;
mod lang;
mod error;

fn read_file_and_parse(path: &str) -> ast::Project {
    let source = std::fs::read_to_string(path).unwrap();
    let project = parser::parse(&source).unwrap();
    // println!("{:#?}", project);
    project
}

use clap::Parser as CliParser;

#[derive(CliParser)]
struct IozhCli {
    #[arg(num_args = 2..)]
    folders: Vec<PathBuf>,
}

fn main() {
    let p = read_file_and_parse("src/tgbot.iozh");
    let s = p.generate(
        std::path::Path::new("/home/sherzod/work/iozh-proj/scala2-test/src/main/scala"),
    );
    println!("{:#?}", s);
}