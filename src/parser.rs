//! Module for parsing the SQL and then using the resulting AST to walk back and
//! check for comments
use std::path::PathBuf;

use sqlparser::{ast::Statement, dialect::GenericDialect, parser::Parser};

use crate::files::{SqlFile, SqlFileSet};

/// A single SQL file plus all [`Statement`].
pub struct ParsedSqlFile {
    file: SqlFile,
    statements: Vec<Statement>,
}
