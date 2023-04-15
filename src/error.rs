use crate::ast;

#[derive(Debug)]
pub struct IozhError {
    pub pos: ast::Pos,
    pub msg: String,
}