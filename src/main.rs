use std::path::{Path, PathBuf};

extern crate pest;
#[macro_use]
extern crate pest_derive;


mod parser;
mod lang;

fn read_file_and_parse(path: &str) -> parser::Project {
    let source = std::fs::read_to_string(path).unwrap();
    let project = parser::parse(&source).unwrap();
    project
}

use clap::Parser as CliParser;

#[derive(CliParser)]
struct IozhCli {
    #[arg(num_args = 2..)]
    folders: Vec<PathBuf>,
}

pub fn run() {
    let args = IozhCli::parse();

    let folders: Vec<&Path> = args.folders
        .iter()
        .map(|p| p.as_path())
        .collect();

    let in_folders = folders.split_last().map(|a| a.1).unwrap_or(&[]).iter().collect::<Vec<_>>();
    let out_folder: &Path = folders.last().unwrap();

    println!("{:#?}", in_folders);
    println!("{:#?}", out_folder);
}

fn main() {
    let p = read_file_and_parse("src/test.iozh");
    let s = p.generate(
        std::path::Path::new("/home/sherzod/work/iozh-proj/scala2-test/src/main/scala"),
    );
    println!("{:#?}", s);
}