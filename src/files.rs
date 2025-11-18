//! The module for working with and loading the content from `sql` files
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct SqlFilesList {
    sql_files: Vec<PathBuf>,
}

impl SqlFilesList {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<SqlFilesList> {
        let recursive_scan = recursive_dir_scan(path.as_ref())?;
        Ok(SqlFilesList {
            sql_files: recursive_scan,
        })
    }
}

/// Helper function to recursively scan the specified directory and collect all sql files found
fn recursive_dir_scan(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut sql_files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap() == "sql" {
            sql_files.push(path);
        } else if path.is_dir() {
            let nested = recursive_dir_scan(&path)?;
            sql_files.extend(nested);
        }
    }
    Ok(sql_files)
}

pub struct SqlFile {
    path: PathBuf,
    content: String,
}

pub struct SqlFileSet {
    files_contents: Vec<SqlFile>
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_recursive_scan_finds_only_sql_files_recursively() {
        // Create a unique temporary base directory
        let base = env::temp_dir().join("recursive_scan_test");
        // Clean up any previous leftovers
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();

        // Create a nested subdirectory
        let sub = base.join("subdir");
        fs::create_dir_all(&sub).unwrap();

        // Paths for SQL files
        let file1 = base.join("one.sql");
        let file2 = sub.join("two.sql");

        // Non-SQL files (should be ignored)
        let non_sql1 = base.join("ignore.txt");
        let non_sql2 = sub.join("README.md");

        // Create the files to be tested
        fs::File::create(&file1).unwrap();
        fs::File::create(&file2).unwrap();
        fs::File::create(&non_sql1).unwrap();
        fs::File::create(&non_sql2).unwrap();

        // Call the function under test
        let mut found = recursive_dir_scan(base.as_path()).unwrap();

        // Sort so order doesn't matter
        found.sort();

        let mut expected = vec![file1, file2];
        expected.sort();

        assert_eq!(found, expected);

        // Optional cleanup
        let _ = fs::remove_dir_all(&base);
    }
}
