//! Module for the top level `SqlDoc` structure. 

use std::path::{Path, PathBuf};

use crate::{
    ast::ParsedSqlFileSet,
    comments::Comments,
    docs::{SqlFileDoc, TableDoc, ColumnDoc},
    error::DocError,
    files::SqlFileSet,
};

pub struct SqlDoc {
    // e.g. all tables across all files
    tables: Vec<TableDoc>,           // or a map keyed by (schema, name)
    // optionally: keep per-file docs too
    files: Vec<(PathBuf, SqlFileDoc)>,
}

pub struct SqlDocBuilder {
    source: SqlFileDocource,
    deny: Vec<String>,
}

enum SqlFileDocource {
    Dir(PathBuf),
    File(PathBuf),
}

impl SqlDoc {
    pub fn from_dir<P: AsRef<Path>>(root: P) -> SqlDocBuilder {
        SqlDocBuilder {
            source: SqlFileDocource::Dir(root.as_ref().to_path_buf()),
            deny: Vec::new(),
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> SqlDocBuilder {
        SqlDocBuilder {
            source: SqlFileDocource::File(path.as_ref().to_path_buf()),
            deny: Vec::new(),
        }
    }

    // later:
    // pub fn table(&self, name: &str) -> Result<&TableDoc, DocError> { ... }
    // pub fn table_with_schema(&self, schema: &str, name: &str) -> Result<&TableDoc, DocError> { ... }
}

impl SqlDocBuilder {
    pub fn deny<S: AsRef<str>>(mut self, pattern: S) -> Self {
        self.deny.push(pattern.as_ref().to_string());
        self
    }

    pub fn build(self) -> Result<SqlDoc, DocError> {
        match self.source {
            SqlFileDocource::Dir(root) => {
                // Use your existing directory logic here
            }
            SqlFileDocource::File(path) => {
                // Same plumbing but for a single file
            }
        }
    }
}
