use regex::Regex;
use scrivenwright::app::AppResult;
use scrivenwright::text::{KeyPress, TestResult};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
};

static SW_DIR: &str = "scrivenwright";

fn sw_dir() -> PathBuf {
    dirs::home_dir().expect("no home dir").join(SW_DIR)
}

fn book_file(book_title: &str) -> PathBuf {
    sw_dir().join(format!("{}.txt", book_title))
}

fn book_dir(book_title: &str) -> PathBuf {
    sw_dir().join(book_title)
}

fn test_file(book_title: &str) -> PathBuf {
    book_dir(book_title).join("tests.json")
}

fn keypress_file(book_title: &str) -> PathBuf {
    book_dir(book_title).join("keypresses.json")
}

pub fn create_book_dir(book_title: &str) {
    let _ = fs::create_dir(book_dir(book_title));
}

pub fn load_book(book_title: &str) -> AppResult<String> {
    let mut book: String = fs::read_to_string(book_file(book_title))?;
    let rules: Vec<(Regex, &str)> = vec![
        //Remove carriage returns
        (Regex::new(r"\r").unwrap(), ""),
        //Remove trailing newline
        (Regex::new(r"\r?\n$").unwrap(), ""),
        //Remove new lines within paragraphs
        (Regex::new(r"([^\n])\n([^\n])").unwrap(), "$1 $2"),
        //Remove duplicate spaces
        (Regex::new(r"  ").unwrap(), " "),
    ];

    for (re, replacement) in rules {
        book = re.replace_all(&book, replacement).into_owned();
    }
    Ok(book)
}

pub fn load_tests(book_title: &str) -> AppResult<Vec<TestResult>> {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(test_file(book_title))?;

    let reader = BufReader::new(file);
    let mut results = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(test) = serde_json::from_str::<TestResult>(&line) {
            results.push(test);
        }
    }

    Ok(results)
}

pub fn save_test(book_title: &str, test: &TestResult) -> AppResult<()> {
    let mut test_log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(test_file(book_title))
        .unwrap();

    let bytes = serde_json::to_vec(test)?;
    test_log.write_all(&bytes)?;
    test_log.write_all(b"\n")?;
    Ok(())
}

pub fn load_keypresses(book_title: &str) -> AppResult<Vec<KeyPress>> {
    let mut key_press_log = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .open(keypress_file(book_title))
        .expect("Failed to open keypress file");
    let mut string = String::new();
    key_press_log.read_to_string(&mut string)?;
    Ok(serde_json::from_str(&string).unwrap_or(Vec::new()))
}

pub fn save_keypresses(book_title: &str, keypresses: &Vec<KeyPress>) -> AppResult<()> {
    let mut key_press_log = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(keypress_file(book_title))?;
    let bytes = serde_json::to_vec(keypresses)?;
    key_press_log.write(&bytes)?;
    Ok(())
}
