//! main file. for testing. shouldn't be part of crate
use std::fs;
use std::io;

mod files;

fn main() -> io::Result<()> {
    let path = "/home/alex/Projects/sql-docs/sql_files/";

    println!("Contents of directory '{}':", path);
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        println!("  {}", path.display());
    }

    Ok(())
}
