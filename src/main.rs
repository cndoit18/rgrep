use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader, Read},
};

fn main() {
    println!("Hello, world!");
}

enum Globs {
    GlobSet(GlobSet),
    Globs(Vec<String>),
}

fn glob(globs: Globs, dir: &str, recursion: bool) -> Result<Vec<String>, Box<dyn Error>> {
    let globset;
    match globs {
        Globs::GlobSet(v) => globset = v,
        Globs::Globs(paths) => {
            let mut builder = GlobSetBuilder::new();
            for glob in paths {
                builder.add(Glob::new(&glob)?);
            }
            globset = builder.build()?;
        }
    }
    let mut paths = Vec::<String>::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path: String = entry.path().to_string_lossy().into();
        if globset.is_match(&path) {
            paths.push(path.clone());
        }

        if recursion && entry.file_type()?.is_dir() {
            paths.extend(glob(Globs::GlobSet(globset.clone()), &path, recursion)?);
        }
    }
    Ok(paths)
}

fn find(text: impl Read, pattern: &str) -> Result<Vec<(usize, String)>, Box<dyn Error>> {
    let re = Regex::new(pattern)?;
    let mut result = Vec::new();

    for (index, r) in BufReader::new(text).lines().enumerate() {
        let text = r?;
        if re.is_match(&text) {
            result.push((index, text));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn test_glob() {
        let textcases: Vec<(Globs, &str, usize, bool)> = vec![
            (Globs::Globs(vec!["**/main.rs".to_string()]), ".", 1, false),
            (Globs::Globs(vec!["./Cargo.*".to_string()]), ".", 2, false),
        ];
        for (globs, path, len, want_err) in textcases {
            let result = glob(globs, path, true);
            match result {
                Ok(v) => {
                    assert_eq!(v.len(), len);
                    assert!(!want_err);
                }
                Err(_) => {
                    assert!(want_err);
                }
            }
        }
    }

    #[test]
    fn test_find() {
        let textcases: Vec<(&str, &str, usize, bool)> =
            vec![("hello", "hello", 1, false), ("hello", "x", 0, false)];
        for (text, pattern, len, want_err) in textcases {
            let result = find(Cursor::new(text), pattern);
            match result {
                Ok(v) => {
                    assert_eq!(v.len(), len);
                    assert!(!want_err);
                }
                Err(_) => {
                    assert!(want_err);
                }
            }
        }
    }
}
