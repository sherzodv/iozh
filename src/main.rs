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

pub struct TypePath {
    pos: Pos,
    path: Vec<TypeTag>,
}

pub struct Field {
    pos: Pos,
    doc: String,
    name: String,
    type_path: TypePath,
}

#[derive(Debug)]
pub struct Structure {
    pos: Pos,
    doc: String,
    name: TypeTag,
    fields: Vec<Field>,
}

enum ChoiceItem {
    Nil,
    TypeTag{ doc: String, choice: TypeTag },
    Structure(Structure),
}

#[derive(Debug)]
pub struct Choice {
    pos: Pos,
    doc: String,
    name: TypeTag,
    choices: Vec<ChoiceItem>,
}

#[derive(Debug)]
pub struct Method {
    pos: Pos,
    doc:  String,
    name: TypeTag,
    args: Vec<Field>,
    result: TypePath,
}

#[derive(Debug)]
pub struct Service {
    pos: Pos,
    doc: String,
    name: TypeTag,
    methods: Vec<Method>,
}

pub struct MethodRef {
    pos: Pos,
    path: Vec<TypeTag>,
}

pub struct HttpRoutePattern {
    pos: Pos,
    items: Vec<String>,
}

#[derive(Debug)]
pub struct HttpRoute {
    pos: Pos,
    verb: String,
    input: TypePath,
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
        if self.args.len() > 0 {
            let args = self.args.iter().map(|a| format!("{:#?}", a)).collect::<Vec<String>>().join(", ");
            write!(f, "{}[{}]", self.name, args)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_path = self.type_path.path.iter().map(|p| format!("{p:#?}")).collect::<Vec<String>>().join(".");
        if self.doc.len() > 0 {
            writeln!(f, "{}", self.doc)?;
        }
        write!(f, "{}: {:#?}", self.name, type_path)
    }
}

impl fmt::Debug for MethodRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.iter().map(|p| format!("{p:#?}")).collect::<Vec<String>>().join(".");
        write!(f, "{}", path)
    }
}

impl fmt::Debug for ProjectItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectItem::Structure(s) => write!(f, "{:#?}", s),
            ProjectItem::Choice(c) => write!(f, "{:#?}", c),
            ProjectItem::Service(s) => write!(f, "{:#?}", s),
            ProjectItem::HttpService(s) => write!(f, "{:#?}", s),
        }
    }
}

impl fmt::Debug for ChoiceItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChoiceItem::TypeTag{ doc, choice } => {
                if doc.len() > 0 {
                    writeln!(f, "{}", doc)?;
                }
                write!(f, "{:#?}", choice)
            }
            ChoiceItem::Structure(s) => {
                write!(f, "{:#?}", s)
            }
            ChoiceItem::Nil => {
                write!(f, "nil")
            }
        }
    }
}

impl fmt::Debug for HttpRoutePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.items.iter().map(|i| format!("{:#?}", i)).collect::<Vec<String>>().join(", ");
        write!(f, "[{}]", items)
    }
}

impl fmt::Debug for TypePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.iter().map(|p| format!("{p:#?}")).collect::<Vec<String>>().join(".");
        write!(f, "{}", path)
    }
}

fn parse_type_args(pair: Pair<Rule>) -> Vec<TypeTag> {
    let mut args = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::type_tag => {
                args.push(parse_type_tag(pair));
            }
            x => unreachable!("unhandled rule: {:#?}", x)
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
            x => {
                unreachable!("unhandled rule: {:#?}: {:#?}", x, pair);
            }
        }
    }
    TypeTag {
        pos: Pos { line, col },
        name,
        args,
    }
}

fn parse_type_path(pair: Pair<Rule>) -> TypePath {
    let (mut line, mut col) = (0, 0);
    let mut path = Vec::new();
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::type_tag => {
                path.push(parse_type_tag(pair));
            }
            x => println!("unhandled rule: {:#?}", x)
        }
    }
    TypePath {
        pos: Pos { line, col },
        path,
    }
}

fn parse_field(pair: Pair<Rule>) -> Field {
    let (mut line, mut col) = (0, 0);
    let mut doc = String::new();
    let mut name = String::new();
    let mut type_path = TypePath {
        pos: Pos { line: 0, col: 0 },
        path: Vec::new(),
    };
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::doc => {
                doc = pair.as_str().to_string();
            }
            Rule::field_name => {
                name = pair.as_str().to_string();
            }
            Rule::type_path => {
                type_path = parse_type_path(pair);
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    Field {
        pos: Pos { line, col },
        doc,
        name,
        type_path,
    }
}

fn parse_structure(pair: Pair<Rule>) -> Structure {
    let mut name = TypeTag{
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let mut doc = String::new();
    let (mut line, mut col) = (0, 0);
    let mut fields = Vec::new();
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::doc => {
                doc = pair.as_str().to_string();
            }
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
        doc,
        name,
        fields,
    }
}

fn parse_choice_item (pair: Pair<Rule>) -> ChoiceItem {
    let mut parsedDoc = String::new();
    let mut choice = ChoiceItem::Nil;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::doc => {
                parsedDoc = pair.as_str().to_string();
            }
            Rule::type_tag => {
                choice = ChoiceItem::TypeTag{ doc: parsedDoc.clone(), choice: parse_type_tag(pair) };
            }
            Rule::structure => {
                choice = ChoiceItem::Structure(parse_structure(pair));
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    match &mut choice {
        ChoiceItem::TypeTag { doc , .. } => {
            *doc = parsedDoc;
        }
        _ => {}
    }
    return choice;
}

fn parse_choice(pair: Pair<Rule>) -> Choice {
    let mut doc = String::new();
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
            Rule::doc => {
                doc = pair.as_str().to_string();
            }
            Rule::choice_name => {
                for pp in pair.into_inner() {
                    match pp.as_rule() {
                        Rule::type_tag => {
                            name = parse_type_tag(pp);
                        }
                        r => unreachable!("unhandled rule: {:#?}", r),
                    }
                }
            }
            Rule::choice_item => {
                choices.push(parse_choice_item(pair));
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    Choice {
        pos: Pos { line, col },
        doc,
        name,
        choices,
    }
}

fn parse_method(pair: Pair<Rule>) -> Method {
    let mut doc = String::new();
    let mut name: TypeTag = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let mut args = Vec::new();
    let mut result = TypePath {
        pos: Pos { line: 0, col: 0 },
        path: Vec::new(),
    };
    let (mut line, mut col) = (0, 0);
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::doc => {
                doc = pair.as_str().to_string();
            }
            Rule::type_tag => {
                name = parse_type_tag(pair);
            }
            Rule::field => {
                args.push(parse_field(pair));
            }
            Rule::method_result => {
                for p in pair.into_inner() {
                    result = parse_type_path(p);
                }
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    Method {
        pos: Pos { line, col },
        doc,
        name,
        args,
        result,
    }
}

fn parse_service(pair: Pair<Rule>) -> Service {
    let mut doc = String::new();
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
            Rule::doc => {
                doc = pair.as_str().to_string();
            }
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
        doc,
        name,
        methods,
    }
}

fn parse_method_ref(pair: Pair<Rule>) -> MethodRef {
    let mut path = Vec::new();
    let (mut line, mut col) = (0, 0);
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        for pp in pair.into_inner() {
            match pp.as_rule() {
                Rule::type_tag => {
                    path.push(parse_type_tag(pp));
                }
                r => unreachable!("unhandled rule: {:#?}", r),
            }
        }
    }
    MethodRef {
        pos: Pos { line, col },
        path,
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
    let mut input: TypePath = TypePath {
        pos: Pos { line: 0, col: 0 },
        path: Vec::new(),
    };
    let mut pattern: HttpRoutePattern = HttpRoutePattern {
        pos: Pos { line: 0, col: 0 },
        items: Vec::new(),
    };
    let mut method: MethodRef = MethodRef {
        pos: Pos { line: 0, col: 0 },
        path: Vec::new(),
    };
    let mut fields = Vec::new();
    let (mut line, mut col) = (0, 0);
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::http_method => {
                verb = pair.as_str().to_string();
            }
            Rule::type_path => {
                input = parse_type_path(pair);
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