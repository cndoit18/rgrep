use clap::Parser;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
};

const HELP_TEMPLATE: &'static str = "{bin} {version} ({author})

{about}
USAGE:
    {usage}
{all-args}
";

#[derive(Parser)]
#[command(version = "1.0", author = "cndoit18 <cndoit18@outlook.com>", help_template = HELP_TEMPLATE)]
struct Cli {
    pattern: String,
    files: Option<String>,
    #[arg(short = 'r', long = "recursive", default_value = "false")]
    recursive: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut reads: Vec<(Option<String>, Box<dyn Read>)> = Vec::new();
    match cli.files {
        Some(files) => {
            for path in glob(Globs::Globs(vec![files]), ".", cli.recursive)? {
                reads.push((Some(path.to_string()), Box::new(File::open(path)?)))
            }
        }
        None => reads.push((None, Box::new(io::stdin()))),
    }

    for (path, read) in reads {
        for (index, text) in find(read, &cli.pattern)? {
            println!(
                "{}:{index}:{text}",
                if let Some(ref p) = path { p } else { "" }
            )
        }
    }
    Ok(())
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

struct PatternStr(String);

impl Matcher for PatternStr {
    fn is_match(&self, haystack: &str) -> bool {
        haystack.contains(&self.0)
    }
}

impl Matcher for Regex {
    fn is_match(&self, haystack: &str) -> bool {
        self.is_match(haystack)
    }
}

trait Matcher {
    fn is_match(&self, haystack: &str) -> bool;
}

fn find(text: impl Read, pattern: &str) -> Result<Vec<(usize, String)>, Box<dyn Error>> {
    let re: Box<dyn Matcher> = Box::new(Regex::new(pattern)?);
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
            (
                Globs::Globs(vec!["./Cargo.*".to_string()]),
                "notfound",
                1,
                true,
            ),
        ];
        for (globs, path, len, want_err) in textcases {
            let result = glob(globs, path, true);
            match result {
                Ok(v) => {
                    assert_eq!(v.len(), len);
                    assert!(!want_err);
                }
                Err(err) => {
                    assert!(want_err, "{err}");
                }
            }
        }
    }

    #[test]
    fn test_find() {
        let textcases: Vec<(&str, &str, usize, bool)> = vec![
            ("hello", "hello", 1, false),
            ("hello", "x", 0, false),
            ("hello", "hel((lo", 0, true),
        ];
        for (text, pattern, len, want_err) in textcases {
            let result = find(Cursor::new(text), pattern);
            match result {
                Ok(v) => {
                    assert_eq!(v.len(), len);
                    assert!(!want_err);
                }
                Err(err) => {
                    assert!(want_err, "{err}");
                }
            }
        }
    }
}
