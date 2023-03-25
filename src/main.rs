extern crate pest;
#[macro_use]
extern crate pest_derive;

use core::fmt;
use pest::Parser;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "iozh.pest"]
pub struct Iozh;

pub struct Pos {
    line: usize,
    col: usize,
}

pub struct TypeTag {
    pos: Pos,
    name: String,
    args: Vec<TypeTag>,
}

pub struct Field {
    pos: Pos,
    name: String,
    type_tag: TypeTag,
}

#[derive(Debug)]
pub struct Structure {
    pos: Pos,
    name: TypeTag,
    fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Choice {
    pos: Pos,
    name: TypeTag,
    choices: Vec<TypeTag>,
}

#[derive(Debug)]
pub struct Project {
    pos: Pos,
    structures: Vec<Structure>,
    choices: Vec<Choice>,
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.line, self.col)
    }
}

impl fmt::Debug for TypeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{:#?}", self.name, self.args)
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:#?}", self.name, self.type_tag)
    }
}

fn parse_type_args(pair: Pair<Rule>) -> Vec<TypeTag> {
    let mut args = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::type_tag => {
                args.push(parse_type_tag(pair));
            }
            x => println!("unhandled rule: {:?}", x)
        }
    }
    args
}

fn parse_type_tag(pair: Pair<Rule>) -> TypeTag {
    let (mut line, mut col) = (0, 0);
    let mut name = String::new();
    let mut args = Vec::new();
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::type_name => {
                name = pair.as_str().to_string();
            }
            Rule::type_args => {
                args = parse_type_args(pair);
            }
            x => println!("unhandled rule: {:?}", x)
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
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::field_name => {
                name = pair.as_str().to_string();
            }
            Rule::type_tag => {
                type_tag = parse_type_tag(pair);
            }
            r => unreachable!("unhandled rule: {:?}", r),
        }
    }
    Field {
        pos: Pos { line, col },
        name,
        type_tag,
    }
}

fn parse_structure(pair: Pair<Rule>) -> Structure {
    let mut name = TypeTag{
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let (mut line, mut col) = (0, 0);
    let mut fields = Vec::new();
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::type_tag => {
                name = parse_type_tag(pair);
            }
            Rule::field => {
                fields.push(parse_field(pair));
            }
            r => unreachable!("unhandled rule: {:?}", r),
        }
    }
    Structure {
        pos: Pos { line, col },
        name,
        fields,
    }
}

fn parse_choice(pair: Pair<Rule>) -> Choice {
    let mut name: TypeTag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let (mut line, mut col) = (0, 0);
    let mut choices = Vec::new();

    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::type_tag => {
                name = parse_type_tag(pair);
            }
            Rule::choice_item => {
                let mut choice_inner_items = parse_type_args(pair);
                choices.append(choice_inner_items.as_mut());
            }
            r => unreachable!("unhandled rule: {:?}", r),
        }
    }
    Choice {
        pos: Pos { line, col },
        name,
        choices,
    }
}

fn parse_project(source: &str) -> Result<Project, Error<Rule>> {
    let projects: Pairs<Rule> = Iozh::parse(Rule::project, source)?;
    let mut structures: Vec<Structure> = Vec::new();
    let mut choices: Vec<Choice> = Vec::new();
    projects.for_each(|project| {
        let items = project.into_inner();
        items.for_each(|item| {
            match item.as_rule() {
                Rule::structure => {
                    structures.push(parse_structure(item));
                }
                Rule::choice => {
                    choices.push(parse_choice(item));
                }
                r => unreachable!("unhandled rule: {:?}", r),
            }
        });
    });
    Ok(Project {
        pos: Pos { line: 0, col: 0 },
        structures,
        choices,
    })
}

fn read_file_and_parse() {
    let source = std::fs::read_to_string("src/test.iozh").unwrap();
    let project = parse_project(&source).unwrap();
    println!("{:#?}", project);
}

fn main() {
    read_file_and_parse();
}