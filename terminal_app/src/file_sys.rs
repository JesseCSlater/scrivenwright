use deunicode::deunicode;
use regex::Regex;
use scrivenwright::app::{AppResult, KeyPress, TestResult};
use std::{fs, io::Read, io::Write, path::PathBuf};

static SW_DIR: &str = "scrivenwright";

fn sw_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(SW_DIR)
}

fn book_file(book_title: &str) -> PathBuf {
    sw_dir().join(format!("{}.txt", book_title))
}

fn book_dir(book_title: &str) -> PathBuf {
    sw_dir().join(book_title)
}

fn test_dir(book_title: &str) -> PathBuf {
    book_dir(book_title).join("tests.json")
}

fn keypress_dir(book_title: &str) -> PathBuf {
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
    let mut test_log = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(test_dir(book_title))
        .expect("Failed to open tests file");
    let mut string = String::new();
    test_log.read_to_string(&mut string)?;
    Ok(serde_json::from_str(&string).unwrap_or(Vec::new()))
}

pub fn save_tests(book_title: &str, tests: &Vec<TestResult>) -> AppResult<()> {
    let mut test_log = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(test_dir(book_title))?;
    let bytes = serde_json::to_vec(tests)?;
    test_log.write(&bytes)?;
    Ok(())
}

pub fn load_keypresses(book_title: &str) -> AppResult<Vec<KeyPress>> {
    let mut key_press_log = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .open(keypress_dir(book_title))
        .expect("Failed to open keypress file");
    let mut string = String::new();
    key_press_log.read_to_string(&mut string)?;
    Ok(serde_json::from_str(&string).unwrap_or(Vec::new()))
}

pub fn save_keypresses(book_title: &str, keypresses: &Vec<KeyPress>) -> AppResult<()> {
    let mut key_press_log = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(keypress_dir(book_title))?;
    let bytes = serde_json::to_vec(keypresses)?;
    key_press_log.write(&bytes)?;
    Ok(())
}
