use crate::ast;

#[derive(Debug)]
pub struct IozhError {
    pub pos: ast::Pos,
    pub msg: String,
}

impl From<String> for IozhError {
    fn from(value: String) -> Self {
        IozhError { pos: ast::Pos::default(), msg: value }
    }
}