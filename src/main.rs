use regex::Regex;
use std::{
    error::Error,
    io::{BufRead, BufReader, Read},
};

fn main() {
    println!("Hello, world!");
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
                    assert!(!want_err);
                }
            }
        }
    }
}
