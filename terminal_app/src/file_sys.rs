use deunicode::deunicode;
use regex::Regex;
use scrivenwright::app::{AppResult, KeyPress, Test};
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
    Ok(deunicode(
        &Regex::new(r"\s+")
            .unwrap()
            .replace_all(
                &fs::read_to_string(book_file(book_title))?
                .trim(),
                " ",
            )
            .to_string(),
    ))
}

pub fn load_tests(book_title: &str) -> AppResult<Vec<Test>> {
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

pub fn save_tests(book_title: &str, tests: &Vec<Test>) -> AppResult<()> {
    let mut test_log = fs::OpenOptions::new().create(true).write(true).open(test_dir(book_title))?;
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
    let mut key_press_log = fs::OpenOptions::new().create(true).write(true).open(keypress_dir(book_title))?;
    let bytes = serde_json::to_vec(keypresses)?;
    key_press_log.write(&bytes)?;
    Ok(())
}
