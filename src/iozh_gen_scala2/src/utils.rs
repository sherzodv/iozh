use iozh_parse::ast;
use iozh_parse::error::IozhError;

use crate::gen::GenResult;

pub trait ResultVec {
    fn join(&self, sep: &str) -> String;
    fn to_string(&self) -> String;
}

pub trait ResultExt<T> {
    fn to_iozh(self) -> Result<T, IozhError>;
}


impl <T> ResultExt<T> for std::io::Result<T> {
    fn to_iozh(self) -> Result<T, IozhError> {
        self.map_err(|e| IozhError {
            pos: ast::Pos { line: 0, col: 0},
            msg: format!("Failed to write file or dir: {}", e),
        })
    }
}

impl ResultVec for Vec<GenResult> {
    fn join(&self, sep: &str) -> String {
        self.iter().map(|x| x.content.clone()).collect::<Vec<_>>().join(sep)
    }
    fn to_string(&self) -> String {
        self.join("")
    }
}