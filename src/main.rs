//! main file. for testing. shouldn't be part of crate
use std::{error::Error, path::Path};

use sqlparser::{dialect::GenericDialect, parser::Parser};

pub mod files;
pub mod parser;
fn main() -> Result<(), Box<dyn Error>> {
    let path: &Path = Path::new("/home/alex/Projects/sql-docs/sql_files/");

    let sql_file_set = files::SqlFileSet::new(path, None)?;

    for sql in sql_file_set.iter() {
        let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...

        let ast = Parser::parse_sql(&dialect, sql.content()).unwrap();

        println!("AST: {ast:?}");
    }

    Ok(())
}
