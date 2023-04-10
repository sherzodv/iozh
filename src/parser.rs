use core::fmt;
use itertools::Itertools;
use pest::Parser;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "iozh.pest"]
pub struct Iozh;

#[derive(Clone)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String{ pos: Pos, value: String },
    Int{ pos: Pos, value: i64 },
    Nil,
}

pub struct TypeTag {
    pub pos: Pos,
    pub name: String,
    pub args: Vec<TypeTag>,
}

pub struct TypePath {
    pub pos: Pos,
    pub path: Vec<TypeTag>,
}

#[derive(Clone)]
pub struct Tag {
    pub pos: Pos,
    pub name: String,
    pub value: Literal,
}

pub struct Field {
    pub pos: Pos,
    pub doc: String,
    pub name: String,
    pub type_path: TypePath,
}

pub enum StructItem {
    Field(Field),
    Tag(Tag),
}

#[derive(Debug)]
pub struct Structure {
    pub pos: Pos,
    pub doc: String,
    pub name: TypeTag,
    pub fields: Vec<StructItem>,
}

pub enum ChoiceItem {
    Nil,
    TypeTag{ doc: String, choice: TypeTag },
    Structure(Structure),
    Value{ doc: String, name: TypeTag, value: Literal },
    Wrap{ doc: String, name: TypeTag, field: String, target: TypePath },
}

#[derive(Debug)]
pub struct Choice {
    pub pos: Pos,
    pub doc: String,
    pub name: TypeTag,
    pub choices: Vec<ChoiceItem>,
}

#[derive(Debug)]
pub struct Method {
    pub pos: Pos,
    pub doc:  String,
    pub name: TypeTag,
    pub args: Vec<Field>,
    pub result: TypePath,
}

#[derive(Debug)]
pub struct Service {
    pub pos: Pos,
    pub doc: String,
    pub name: TypeTag,
    pub methods: Vec<Method>,
}

pub struct MethodRef {
    pub pos: Pos,
    pub path: Vec<TypeTag>,
}

pub struct HttpRoutePattern {
    pub pos: Pos,
    pub items: Vec<String>,
}

#[derive(Debug)]
pub struct HttpRoute {
    pub pos: Pos,
    pub verb: String,
    pub input: TypePath,
    pub pattern: HttpRoutePattern,
    pub method: MethodRef,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct HttpService {
    pub pos: Pos,
    pub name: TypeTag,
    pub routes: Vec<HttpRoute>,
}

pub enum NspaceItem {
    Structure(Structure),
    Choice(Choice),
    Service(Service),
    HttpService(HttpService),
    Nspace(Nspace),
}

#[derive(Debug)]
pub struct Nspace {
    pub pos: Pos,
    pub name: String,
    pub items: Vec<NspaceItem>,
}

#[derive(Debug)]
pub struct Project {
    pub pos: Pos,
    pub nspaces: Vec<Nspace>,
}

impl Tag {
    pub fn get_value_as_str(&self) -> String {
        match &self.value {
            Literal::String{ pos: _, value } => value.clone(),
            Literal::Int{ pos: _, value } => value.to_string(),
            Literal::Nil => "nil".to_string(),
        }
    }
}

impl ChoiceItem {
    pub fn get_tag_value(&self, tag: &str) -> String {
        match self {
            ChoiceItem::Structure(s) => {
                if let Some(tag_val) = s.get_tag(tag).map(|t| t.get_value_as_str()) {
                    tag_val
                } else {
                    s.name.name.clone()
                }
            }
            _ => "WRONG_PLACE_TO_USE_TAG".to_string()
        }
    }
    pub fn get_tags(&self) -> Vec<Tag> {
        match self {
            ChoiceItem::Structure(s) => s.get_tags(),
            _ => vec![],
        }
    }
}

impl Choice {
    pub fn get_most_common_tag_key(&self) -> Option<String> {
        let counts = self.choices
            .iter()
            .map(|c| c.get_tags())
            .flatten()
            .map(|t| t.name)
            .sorted()
            .counts();
        if counts.len() > 0 {
            counts.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(name, _)| name.clone())
        } else {
            None
        }
    }
}

impl Structure {
    fn get_tag(&self, tag: &str) -> Option<Tag> {
        self.fields.iter().find_map(|field| match field {
            StructItem::Tag(t) if t.name == tag => Some(t.clone()),
            _ => None,
        })
    }
    fn get_tags(&self) -> Vec<Tag> {
        self.fields
            .iter()
            .filter_map(|field| match field {
                StructItem::Tag(t) => Some(t.clone()),
                _ => None,
            })
            .collect::<Vec<Tag>>()
    }
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

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Literal::String{ pos: _, value } => write!(f, "{}: \"{}\"", self.name, value),
            Literal::Int{ pos: _, value } => write!(f, "{}: {}", self.name, value),
            Literal::Nil => write!(f, "{}: nil", self.name),
        }
    }
}

impl fmt::Debug for StructItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StructItem::Field(field) => write!(f, "{:#?}", field),
            StructItem::Tag(t) => write!(f, "{:#?}", t),
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

impl fmt::Debug for NspaceItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NspaceItem::Structure(s) => write!(f, "{:#?}", s),
            NspaceItem::Choice(c) => write!(f, "{:#?}", c),
            NspaceItem::Service(s) => write!(f, "{:#?}", s),
            NspaceItem::HttpService(s) => write!(f, "{:#?}", s),
            NspaceItem::Nspace(n) => write!(f, "{:#?}", n),
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
            ChoiceItem::Value{ doc: _, name, value} => {
                write!(f, "{:#?} = {:#?}", name, value)
            }
            ChoiceItem::Nil => {
                write!(f, "nil")
            }
            ChoiceItem::Wrap { doc, name, field, target } => {
                if doc.len() > 0 {
                    writeln!(f, "{}", doc)?;
                }
                write!(f, "{:#?}({:#?}) = {:#?}", name, field, target)
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

fn parse_literal(pair: Pair<Rule>) -> Literal {
    let mut lit = Literal::Nil;
    for pair in pair.into_inner() {
        let (line, col) = pair.as_span().start_pos().line_col();
        let pos = Pos{ line, col };
        match pair.as_rule() {
            Rule::string_literal => {
                let value = pair.as_str().to_string();
                lit = Literal::String{ pos, value }
            }
            Rule::integer_literal => {
                let value = pair.as_str().trim().parse::<i64>().expect("failed to parse integer literal");
                lit = Literal::Int{ pos, value }
            }
            x => unreachable!("unhandled rule: {:#?}", x)
        }
    }
    lit
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

fn parse_tag(pair: Pair<Rule>) -> Tag {
    let (mut line, mut col) = (0, 0);
    let mut name = String::new();
    let mut value = Literal::Nil;
    for pair in pair.into_inner() {
        (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::field_name => {
                name = pair.as_str().to_string();
            }
            Rule::literal => {
                value = parse_literal(pair);
            }
            x => unreachable!("unhandled rule: {:#?}", x)
        }
    }
    Tag {
        pos: Pos { line, col },
        name,
        value,
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
                fields.push(StructItem::Field(parse_field(pair)));
            }
            Rule::tag => {
                fields.push(StructItem::Tag(parse_tag(pair)));
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

fn parse_choice_item_value(pair: Pair<Rule>) -> ChoiceItem {
    let mut doc = String::new();
    let mut name = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let mut value = Literal::Nil;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::doc => {
                doc = pair.as_str().to_string();
            }
            Rule::type_tag => {
                name = parse_type_tag(pair);
            }
            Rule::literal => {
                value = parse_literal(pair);
            }
            x => unreachable!("unhandled rule: {:#?}", x)
        }
    }
    ChoiceItem::Value { doc, name, value }
}

fn parse_choice_item_wrap(pair: Pair<Rule>) -> ChoiceItem {
    let mut doc = String::new();
    let mut name = TypeTag {
        pos: Pos { line: 0, col: 0 },
        name: String::new(),
        args: Vec::new(),
    };
    let mut field = String::new();
    let mut target = TypePath {
        pos: Pos { line: 0, col: 0 },
        path: Vec::new(),
    };
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::doc => {
                doc = pair.as_str().to_string();
            }
            Rule::type_tag => {
                name = parse_type_tag(pair);
            }
            Rule::type_name => {
                field = pair.as_str().to_string();
            }
            Rule::type_path => {
                target = parse_type_path(pair);
            }
            x => unreachable!("unhandled rule: {:#?}", x)
        }
    }
    ChoiceItem::Wrap { doc, name, field, target }
}

fn parse_choice_item (pair: Pair<Rule>) -> ChoiceItem {
    let mut parsed_doc = String::new();
    let mut choice = ChoiceItem::Nil;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::doc => {
                parsed_doc = pair.as_str().to_string();
            }
            Rule::type_tag => {
                choice = ChoiceItem::TypeTag{ doc: parsed_doc.clone(), choice: parse_type_tag(pair) };
            }
            Rule::structure => {
                choice = ChoiceItem::Structure(parse_structure(pair));
            }
            Rule::choice_item_value => {
                choice = parse_choice_item_value(pair);
            }
            Rule::choice_item_wrap => {
                choice = parse_choice_item_wrap(pair);
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    }
    match &mut choice {
        ChoiceItem::TypeTag { doc , .. } => {
            *doc = parsed_doc;
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
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut routes = Vec::new();
    for pair in pair.into_inner() {
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

fn parse_namespace(pair: Pair<Rule>) -> Nspace {
    let mut name = String::new();
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut items: Vec<NspaceItem> = Vec::new();
    pair.into_inner().for_each(|entity| {
        match entity.as_rule() {
            Rule::nspace_name => {
                name = entity.as_str().to_string();
            }
            Rule::nspace => {
                items.push(NspaceItem::Nspace(parse_namespace(entity)));
            }
            Rule::structure => {
                items.push(NspaceItem::Structure(parse_structure(entity)));
            }
            Rule::choice => {
                items.push(NspaceItem::Choice(parse_choice(entity)));
            }
            Rule::service => {
                items.push(NspaceItem::Service(parse_service(entity)));
            }
            Rule::http_service => {
                items.push(NspaceItem::HttpService(parse_http_service(entity)));
            }
            r => unreachable!("unhandled rule: {:#?}", r),
        }
    });
    Nspace {
        pos: Pos { line, col },
        name,
        items,
    }
}

pub fn parse_project(tree: Pairs<Rule>) -> Project {
    let mut nspaces: Vec<Nspace> = Vec::new();
    tree.for_each(|project| {
        let nss = project.into_inner();
        nss.for_each(|ns| {
            match ns.as_rule() {
                Rule::nspace => {
                    nspaces.push(parse_namespace(ns));
                }
                r => unreachable!("unhandled rule: {:#?}", r),
            }
        })
    });
    Project {
        pos: Pos { line: 0, col: 0 },
        nspaces: nspaces,
    }
}

pub fn parse(source: &str) -> Result<Project, Error<Rule>> {
    let ast: Pairs<Rule> = Iozh::parse(Rule::project, source)?;
    Ok(parse_project(ast))
}