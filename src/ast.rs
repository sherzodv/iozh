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

#[derive(Clone)]
pub struct TypeTag {
    pub pos: Pos,
    pub name: String,
    pub args: Vec<TypePath>,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Field {
    pub pos: Pos,
    pub doc: String,
    pub name: String,
    pub type_path: TypePath,
}

#[derive(Clone)]
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
    pub fields: Vec<Field>,
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