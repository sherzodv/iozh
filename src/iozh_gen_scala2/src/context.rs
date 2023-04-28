use iozh_parse::ast;
use iozh_parse::error::IozhError;

use crate::loc::*;
use crate::gen::*;

#[derive(Debug)]
pub struct ProjectContext<'a> {
    pub p: &'a ast::Project,
}

#[derive(Debug)]
pub struct NspaceContext<'a> {
    pub project: &'a ProjectContext<'a>,
    pub path: Vec<String>,
}

#[derive(Debug)]
pub struct StructContext<'a> {
    pub nspace: &'a NspaceContext<'a>,
    pub base_name: String,
    pub full_type_name: String,
    pub type_args: Vec<String>,
}

pub struct ChoiceContext<'a> {
    pub nspace: &'a NspaceContext<'a>,
    pub p: &'a ast::Choice,
    pub base_name: String,
    pub full_type_name: String,
    pub most_common_tag_key: Option<String>,
}

#[derive(Debug)]
pub struct ServiceContext<'a> {
    pub nspace: &'a NspaceContext<'a>,
    pub base_name: String,
    pub full_type_name: String,
}

pub struct HttpServiceContext<'a> {
    pub nspace: &'a NspaceContext<'a>,
    pub base_name: String,
    pub full_type_name: String,
}

pub struct MethodContext {
    pub name: String,
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