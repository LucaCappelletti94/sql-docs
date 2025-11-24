//! Module for parsing sql and comments and returning only comments connected to
//! statements
use crate::{ast::ParsedSqlFile, comments::Comments};

/// Structure for containing the `name` of the `Column` and an [`Option`] for
/// the comment as a [`String`]
pub struct ColumnDoc {
    name: String,
    doc: Option<String>,
}

/// Structure for containing the `name` of the `Table`, an [`Option`] for the
/// comment as a [`String`], and a `Vec` of [`ColumnDoc`] contained in the table
pub struct TableDoc {
    name: String,
    doc: Option<String>,
    columns: Vec<ColumnDoc>,
}

/// Structure for containing the docs for every `Table` in an `.sql` file as a
/// `Vec` of [`TableDoc`]
pub struct SqlDocs {
    tables: Vec<TableDoc>,
}

impl SqlDocs {
    /// Create a new instance of [`SqlDocs`]
    #[must_use]
    pub fn new(tables: Vec<TableDoc>) -> Self {
        Self { tables }
    }

    /// Builds documentation for sql file by attaching leading comments to tables and columns
    #[must_use]
    pub fn from_parsed_file(file: &ParsedSqlFile, comments: &Comments) -> Self {
        todo!()
    }

    /// Getter function to get a slice of [`TableDoc`]
    #[must_use]
    pub fn tables(&self) -> &[TableDoc] {
        &self.tables
    }
}
