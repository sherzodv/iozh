extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "iozh.pest"]
pub struct Iozh;

#[derive(Debug)]
pub struct Pos {
    line: usize,
    col: usize,
}

#[derive(Debug)]
pub struct TypeTag {
    pos: Pos,
    name: String,
    args: Vec<TypeTag>,
}

#[derive(Debug)]
pub struct Field {
    pos: Pos,
    name: String,
    type_tag: TypeTag,
}

#[derive(Debug)]
pub struct Structure {
    pos: Pos,
    name: String,
    args: Vec<String>,
    fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Project {
    pos: Pos,
    structures: Vec<Structure>,
}

fn parse_type_tag(pair: Pair<Rule>) -> TypeTag {
    let (mut line, mut col) = (0, 0);
    let mut name = String::new();
    let mut args = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::type_tag_name => {
                (line, col) = pair.as_span().start_pos().line_col();
                name = pair.as_str().to_string();
            }
            Rule::type_args => {
                args.push(parse_type_tag(pair));
            }
            _ => unreachable!(),
        }
    }
    TypeTag {
        pos: Pos { line, col },
        name,
        args,
    }
}

fn parse_field(pair: Pair<Rule>) -> Field {
    let (mut line, mut col) = (0, 0);
    let mut name = String::new();
    let mut type_tag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::field_name => {
                (line, col) = pair.as_span().start_pos().line_col();
                name = pair.as_str().to_string();
            }
            Rule::type_tag => {
                type_tag = parse_type_tag(pair);
            }
            _ => unreachable!(),
        }
    }
    Field {
        pos: Pos { line, col },
        name,
        type_tag,
    }
}

fn parse_structure(pair: Pair<Rule>) -> Structure {
    let mut name = String::new();
    let (mut line, mut col) = (0, 0);
    let mut args = Vec::new();
    let mut fields = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::structure_name => {
                (line, col) = pair.as_span().start_pos().line_col();
                name = pair.as_str().to_string();
            }
            Rule::field => {
                fields.push(parse_field(pair));
            }
            _ => unreachable!(),
        }
    }
    Structure {
        pos: Pos { line, col },
        name,
        args,
        fields,
    }
}

fn parse_project(source: &str) -> Result<Project, Error<Rule>> {
    let projects: Pairs<Rule> = Iozh::parse(Rule::project, source)?;
    let mut structures: Vec<Structure> = Vec::new();
    projects.for_each(|project| {
        let structs = project.into_inner();
        structs.for_each(|s| {
            match s.as_rule() {
                Rule::structure => {
                    structures.push(parse_structure(s));
                }
                x => {
                    println!("unhandled rule: {:?}", x);
                }
            }
        });
    });
    Ok(Project {
        pos: Pos { line: 0, col: 0 },
        structures
    })
}

fn debug_print_indented_pair(root: &Pair<Rule>, level: usize) {
    for _ in 0..level {
        print!("  ");
    }
    println!("{:?}", root.as_str());
    for pair in root.clone().into_inner() {
        debug_print_indented_pair(&pair, level + 1);
    }
}

fn debug_print_indented(project: &Project) {
    println!("Project:");
    for structure in &project.structures {
        println!("  Structure ({}, {}): {}", structure.pos.line, structure.pos.col, structure.name);
        for field in &structure.fields {
            println!("    Field({}, {}): {} {}", field.pos.line, field.pos.col, field.name, field.type_tag.name);
        }
    }
}

fn read_file_and_parse() {
    let source = std::fs::read_to_string("src/test.iozh").unwrap();
    let project = parse_project(&source).unwrap();
    debug_print_indented(&project)
}

fn main() {
    read_file_and_parse();
}