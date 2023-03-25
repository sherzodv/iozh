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
enum ChoiceItem {
    TypeTag(TypeTag),
    Structure(Structure),
}

#[derive(Debug)]
pub struct Choice {
    pos: Pos,
    name: TypeTag,
    choices: Vec<ChoiceItem>,
}

#[derive(Debug)]
pub struct Method {
    pos: Pos,
    name: TypeTag,
    args: Vec<Field>,
    result: TypeTag,
}

#[derive(Debug)]
pub struct Service {
    pos: Pos,
    name: TypeTag,
    methods: Vec<Method>,
}

#[derive(Debug)]
pub struct MethodRef {
    pos: Pos,
    service: TypeTag,
    method: TypeTag,
}

#[derive(Debug)]
pub struct HttpRoutePattern {
    pos: Pos,
    items: Vec<String>,
}

#[derive(Debug)]
pub struct HttpRoute {
    pos: Pos,
    verb: String,
    input: TypeTag,
    pattern: HttpRoutePattern,
    method: MethodRef,
    fields: Vec<Field>,
}

#[derive(Debug)]
pub struct HttpService {
    pos: Pos,
    name: TypeTag,
    routes: Vec<HttpRoute>,   
}

#[derive(Debug)]
enum ProjectItem {
    Structure(Structure),
    Choice(Choice),
    Service(Service),
    HttpService(HttpService),
}

#[derive(Debug)]
pub struct Project {
    pos: Pos,
    items: Vec<ProjectItem>,
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
            x => println!("unhandled rule: {:#?}", x)
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
            x => println!("unhandled rule: {:#?}", x)
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
            r => unreachable!("unhandled rule: {:#?}", r),
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
            r => unreachable!("unhandled rule: {:#?}", r),
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
            Rule::structure => {
                choices.push(ChoiceItem::Structure(parse_structure(pair)));
            }
            Rule::choice_item => {
                let mut choice_inner_items = parse_type_args(pair);
                for item in choice_inner_items.drain(..) {
                    choices.push(ChoiceItem::TypeTag(item));
                }
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    Choice {
        pos: Pos { line, col },
        name,
        choices,
    }
}

fn parse_method(pair: Pair<Rule>) -> Method {
    let mut name: TypeTag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let mut args = Vec::new();
    let mut result = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let (mut line, mut col) = (0, 0);
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::type_tag => {
                name = parse_type_tag(pair);
            }
            Rule::field => {
                args.push(parse_field(pair));
            }
            Rule::method_result => {
                result = parse_type_tag(pair);
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    Method {
        pos: Pos { line, col },
        name,
        args,
        result,
    }
}

fn parse_service(pair: Pair<Rule>) -> Service {
    let mut name: TypeTag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let (mut line, mut col) = (0, 0);
    let mut methods = Vec::new();
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::type_tag => {
                name = parse_type_tag(pair);
            }
            Rule::method => {
                methods.push(parse_method(pair));
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    Service {
        pos: Pos { line, col },
        name,
        methods,
    }
}

fn parse_method_ref(pair: Pair<Rule>) -> MethodRef {
    let mut service: TypeTag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let mut method: TypeTag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let (mut line, mut col) = (0, 0);
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::service_ref => {
                service = parse_type_tag(pair.into_inner().next().unwrap());
            }
            Rule::type_tag => {
                method = parse_type_tag(pair);
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    MethodRef {
        pos: Pos { line, col },
        service,
        method,
    }
}

fn parse_http_route_pattern(pair: Pair<Rule>) -> HttpRoutePattern {
    let (mut line, mut col) = (0, 0);
    let mut items = Vec::new();
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::http_route_var => {
                items.push(pair.as_str().to_string());
            }
            Rule::http_path_part => {
                items.push(pair.as_str().to_string());
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    HttpRoutePattern {
        pos: Pos { line, col },
        items,
    }
}

fn parse_http_route(pair: Pair<Rule>) -> HttpRoute {
    let mut verb: String = String::new();
    let mut input: TypeTag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let mut pattern: HttpRoutePattern = HttpRoutePattern {
        pos: Pos { line: 0, col: 0 },
        items: Vec::new(),
    };
    let mut method: MethodRef = MethodRef {
        pos: Pos { line: 0, col: 0 },
        service: TypeTag {
            pos: Pos { line: 0, col: 0 },
            name: String::new(),
            args: Vec::new(),
        },
        method: TypeTag {
            pos: Pos { line: 0, col: 0 },
            name: String::new(),
            args: Vec::new(),
        },
    };
    let mut fields = Vec::new();
    let (mut line, mut col) = (0, 0);
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::http_method => {
                verb = pair.as_str().to_string();
            }
            Rule::type_tag => {
                input = parse_type_tag(pair);
            }
            Rule::http_route_pattern => {
                pattern = parse_http_route_pattern(pair);
            }
            Rule::method_ref => {
                method = parse_method_ref(pair);
            }
            Rule::field => {
                fields.push(parse_field(pair));
            } 
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    HttpRoute {
        pos: Pos { line, col },
        verb,
        input,
        pattern,
        method,
        fields,
    }
}

fn parse_http_service(pair: Pair<Rule>) -> HttpService {
    let mut name: TypeTag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let (mut line, mut col) = (0, 0);
    let mut routes = Vec::new();
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::type_tag => {
                name = parse_type_tag(pair);
            }
            Rule::http_route => {
                routes.push(parse_http_route(pair));
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    HttpService {
        pos: Pos { line, col },
        name,
        routes,
    }
}

fn parse_project(source: &str) -> Result<Project, Error<Rule>> {
    let projects: Pairs<Rule> = Iozh::parse(Rule::project, source)?;
    let mut name = String::new();
    let (mut line, mut col) = (0, 0);
    let mut items: Vec<ProjectItem> = Vec::new();
    projects.for_each(|project| {
        let entities = project.into_inner();
        entities.for_each(|entity| {
            match entity.as_rule() {
                Rule::structure => {
                    items.push(ProjectItem::Structure(parse_structure(entity)));
                }
                Rule::choice => {
                    items.push(ProjectItem::Choice(parse_choice(entity)));
                }
                Rule::service => {
                    items.push(ProjectItem::Service(parse_service(entity)));
                }
                Rule::http_service => {
                    items.push(ProjectItem::HttpService(parse_http_service(entity)));
                }
                r => unreachable!("unhandled rule: {:#?}", r),
            }
        })
    });
    Ok(Project {
        pos: Pos { line: 0, col: 0 },
        items,
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