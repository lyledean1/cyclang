use std::fmt;
use std::error::Error;

use crate::parser::Rule;

#[derive(Debug)]
pub enum CycloError {
    CompileError(std::io::Error),
    ParseError(Box<pest::error::Error<Rule>>),
}

impl fmt::Display for CycloError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CycloError::CompileError(err) => write!(f, "Error compiling to LLVM: {}", err),
            CycloError::ParseError(err) => write!(f, "Error parsing to AST: {}", err),
        }
    }
}

impl Error for CycloError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CycloError::CompileError(err) => Some(err),
            CycloError::ParseError(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for CycloError {
    fn from(err: std::io::Error) -> CycloError {
        CycloError::CompileError(err)
    }
}

impl From<Box<pest::error::Error<Rule>>> for CycloError {
    fn from(err: Box<pest::error::Error<Rule>>) -> CycloError {
        CycloError::ParseError(err)
    }
}
