//! Module for parsing the SQL and then using the resulting AST to walk back and
//! check for comments
use std::path::Path;

use sqlparser::{
    ast::Statement,
    dialect::GenericDialect,
    parser::{Parser, ParserError},
};

use crate::files::{SqlFile, SqlFileSet};

/// A single SQL file plus all [`Statement`].
pub struct ParsedSqlFile {
    file: SqlFile,
    statements: Vec<Statement>,
}

impl ParsedSqlFile {
    /// Parses all statements from the specified sql file
    ///
    /// # Parameters
    /// - `file` is the [`SqlFile`] that will be parsed
    ///
    /// # Errors
    /// - [`ParserError`] is returned for any errors parsing
    pub fn parse(file: SqlFile) -> Result<Self, ParserError> {
        let dialect = GenericDialect {};
        let statements = Parser::parse_sql(&dialect, file.content())?;
        Ok(Self { file, statements })
    }

    /// Getter method for returning the current object's file's path
    #[must_use]
    pub fn path(&self) -> &Path {
        self.file.path()
    }

    /// Getter method for returning the vector of all statements [`Statement`]
    #[must_use]
    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }
}

/// Struct to contain the vector of parsed SQL files
pub struct ParsedSqlFileSet {
    files: Vec<ParsedSqlFile>,
}

impl ParsedSqlFileSet {
    /// Method that parses all members in a [`SqlFileSet`]
    ///
    /// # Parameters
    /// - `set` the set of [`SqlFileSet`]
    ///
    /// # Errors
    /// - [`ParserError`] is returned for any errors parsing
    pub fn parse_all(set: SqlFileSet) -> Result<Self, ParserError> {
        let files = set.into_iter().map(ParsedSqlFile::parse).collect::<Result<Vec<_>, _>>()?;

        Ok(Self { files })
    }

    /// Getter method for returning the vector of all [`ParsedSqlFile`]
    #[must_use]
    pub fn files(&self) -> &[ParsedSqlFile] {
        &self.files
    }
}
