use std::path::PathBuf;
use iozh_parse::ast;
use iozh_gen_scala2::gen::generate;

fn read_file_and_parse(path: &str) -> ast::Project {
    let source = std::fs::read_to_string(path).unwrap();
    let project = ast::Project::parse(&source).unwrap();
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
    let p = read_file_and_parse("src/iozh_test/tgbot.iozh");
    let s = generate(p, std::path::Path::new("/home/sherzod/work/iozh-proj/scala2-test/src/main/scala"));
    println!("{:#?}", s);
}