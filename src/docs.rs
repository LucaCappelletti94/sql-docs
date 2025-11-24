//! Module for parsing sql and comments and returning only comments connected to
//! statements
use crate::{ast::ParsedSqlFile, comments::Comments};

/// Structure for containing the `name` of the `Column` and an [`Option`] for
/// the comment as a [`String`]
pub struct ColumnDoc {
    name: String,
    doc: Option<String>,
}
impl ColumnDoc {
    /// Getter for the `name` field
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Getter for the field `doc`
    #[must_use]
    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }
}

/// Structure for containing the `name` of the `Table`, an [`Option`] for the
/// comment as a [`String`], and a `Vec` of [`ColumnDoc`] contained in the table
pub struct TableDoc {
    name: String,
    doc: Option<String>,
    columns: Vec<ColumnDoc>,
}

impl TableDoc {

    /// Getter for the `name` field
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Getter for the `doc` field 
    #[must_use]
    pub fn doc(&self) -> &Option<String> {
        &self.doc
    }

    /// Getter for the `columns` field
    #[must_use]
    pub fn columns(&self) -> &[ColumnDoc] {
        &self.columns
    }
}

/// Structure for containing the docs for every `Table` in an `.sql` file as a
/// `Vec` of [`TableDoc`]
pub struct SqlDocs {
    tables: Vec<TableDoc>,
}

impl SqlDocs {
    /// Create a new instance of [`SqlDocs`]
    #[must_use]
    pub const fn new(tables: Vec<TableDoc>) -> Self {
        Self { tables }
    }

    /// Builds documentation for sql file by attaching leading comments to
    /// tables and columns
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
