mod utils;
mod loc;
pub mod gen;

use std::path::PathBuf;
use crate::parser as p;
use crate::lang::scala2::loc::Loc;

#[derive(Debug, Clone)]
pub struct ProjectContext {
    target_folder: PathBuf,
}

#[derive(Debug, Clone)]
pub struct NspaceContext {
    project: ProjectContext,
    folder: PathBuf,
    path: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StructContext {
    pub nspace: NspaceContext,
    pub base_name: String,
    pub full_type_name: String,
}

pub struct ChoiceContext {
    nspace: NspaceContext,
    base_name: String,
    full_type_name: String,
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
    pub pos: p::Pos,
    pub msg: String,
}

pub struct GenResult {
    file: Option<PathBuf>,
    content: String,
    imports: Vec<String>,
    package: Vec<String>,
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