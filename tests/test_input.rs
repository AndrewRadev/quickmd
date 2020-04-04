use std::path::PathBuf;
use std::fs;

use claim::assert_matches;
use quickmd::input::InputFile;

#[test]
fn test_input_file_constructed_from_local_file() {
    let path = PathBuf::from("./test.md");
    let buffer = Vec::<u8>::new();
    let input_file = InputFile::from(&path, buffer.as_slice()).unwrap();

    assert_matches!(input_file, InputFile::Filesystem(_));
    assert_eq!("./test.md", input_file.path().to_str().unwrap());
}

#[test]
fn test_input_file_constructed_from_stdin() {
    let path = PathBuf::from("-");
    let buffer = Vec::<u8>::new();
    let input_file = InputFile::from(&path, buffer.as_slice()).unwrap();

    assert_matches!(input_file, InputFile::Stdin(_));
    // Not really "-", would be a temporary file:
    assert_ne!("-", input_file.path().to_str().unwrap());
}

#[test]
fn test_reading_from_file_built_from_stdin() {
    let path = PathBuf::from("-");
    let buffer: Vec<u8> = "# Test\nTest content".bytes().collect();
    let input_file = InputFile::from(&path, buffer.as_slice()).unwrap();

    assert_eq!(b"# Test\nTest content", fs::read(input_file.path()).unwrap().as_slice());
}

#[test]
fn test_temporary_file_is_cleaned_up_on_drop() {
    let path = PathBuf::from("-");
    let buffer: Vec<u8> = "# Test\nTest content".bytes().collect();
    let input_file = InputFile::from(&path, buffer.as_slice()).unwrap();
    let path = input_file.path().to_path_buf();

    assert!(path.exists());
    drop(input_file);
    assert!(!path.exists());
}
