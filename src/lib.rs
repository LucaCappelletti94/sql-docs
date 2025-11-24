//! High-level library entrypoint for the SQL docs tool.
//!
//! Module layout:
//! - [`files`]    : discover and load `.sql` files from disk
//! - [`ast`]      : parse SQL into an AST using `sqlparser`
//! - [`comments`] : extract and model comments + spans
//!
//! Most users should be able to use the convenience functions
//! [`parse_comments_from_path`] and [`parse_comments_from_dir`].

pub mod files;
pub mod ast;
pub mod comments;

use std::{io, path::{Path, PathBuf}};

use ast::{ParsedSqlFile, ParsedSqlFileSet};
use comments::{CommentError, Comments};
use files::SqlFileSet;
use sqlparser::parser::ParserError;

/// Unified error types
///
/// Wraps:
/// - filesystem errors (`io::Error`)
/// - SQL parsing errors (`sqlparser::parser::ParserError`)
/// - comment extraction errors (`comments::CommentError`)
#[derive(Debug)]
pub enum Error {
    /// `std::io::Error` for i/o errors
    Io(io::Error),
    /// `SqlParse` error for errors with the SQL parsing
    SqlParse(ParserError),
    /// `Comments` for [`CommentError`] 
    Comments(CommentError),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ParserError> for Error {
    fn from(err: ParserError) -> Self {
        Self::SqlParse(err)
    }
}

impl From<CommentError> for Error {
    fn from(err: CommentError) -> Self {
        Self::Comments(err)
    }
}
