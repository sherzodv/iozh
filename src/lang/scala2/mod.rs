mod utils;
mod loc;
pub mod gen;
pub mod gen_circe;

use crate::ast;
use crate::lang::scala2::loc::Loc;

pub struct GenResult {
    unit: Option<String>,
    content: String,
    imports: Vec<String>,
    package: Vec<String>,
    block: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
}

#[derive(Debug, Clone)]
pub struct NspaceContext {
    project: ProjectContext,
    path: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StructContext {
    pub nspace: NspaceContext,
    pub base_name: String,
    pub full_type_name: String,
    pub type_args: Vec<String>,
}

pub struct ChoiceContext<'a> {
    p: &'a ast::Choice,
    nspace: NspaceContext,
    base_name: String,
    full_type_name: String,
    most_common_tag_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceContext {
    pub nspace: NspaceContext,
    pub base_name: String,
    pub full_type_name: String,
}

pub struct HttpServiceContext {
    pub nspace: NspaceContext,
    pub base_name: String,
    pub full_type_name: String,
}

pub struct MethodContext {
    pub service: ServiceContext,
    pub name: String,
}


#[derive(Debug)]
pub struct IozhError {
    pub pos: ast::Pos,
    pub msg: String,
}

pub trait Gen where Self: Loc, Self: std::fmt::Debug {
    fn gen(&self) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait InProject where Self: Loc, Self: std::fmt::Debug {
    fn gen_in_project(&self, parent: &ProjectContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait InNspace where Self: Loc, Self: std::fmt::Debug {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait CirceInNspace where Self: Loc, Self: std::fmt::Debug {
    fn decoder_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
    fn encoder_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
    fn codec_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait CirceInStruct where Self: Loc, Self: std::fmt::Debug {
    fn decoder_in_struct(&self, parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError>;
    fn encoder_in_struct(&self, parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait CirceInChoice where Self: Loc, Self: std::fmt::Debug {
    fn decoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
    fn encoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait InStruct where Self: Loc, Self: std::fmt::Debug {
    fn gen_in_struct(&self, parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait InChoice where Self: Loc, Self: std::fmt::Debug {
    fn gen_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait InService where Self: Loc, Self: std::fmt::Debug {
    fn gen_in_service(&self, parent: &ServiceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait InHttpService where Self: Loc, Self: std::fmt::Debug {
    fn gen_in_http_service(&self, parent: &HttpServiceContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}

pub trait InMethod where Self: Loc, Self: std::fmt::Debug {
    fn gen_in_method(&self, parent: &MethodContext) -> std::result::Result<Vec<GenResult>, IozhError>;
}