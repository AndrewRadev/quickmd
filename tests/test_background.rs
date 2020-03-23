use std::fs;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError::Timeout as TimeoutError;
use std::time::Duration;

use claim::assert_matches;

use quickmd::ui;
use quickmd::markdown::Renderer;
use quickmd::background::init_update_loop;

// TODO test for refreshing the page on ~/.quickmd.css change

#[test]
fn test_update_loop_detects_file_updates() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file.md");

    fs::write(&path, "# Test").unwrap();
    let renderer = Renderer::new(path.clone());

    let (sender, receiver) = mpsc::channel();
    init_update_loop(renderer, sender);
    // Wait for the watcher thread to get ready
    std::thread::sleep(Duration::from_millis(10));

    fs::write(path, "# Changed").unwrap();

    // Expect LoadHtml message
    let message = receiver.recv_timeout(Duration::from_millis(300));
    assert_matches!(message, Ok(ui::Event::LoadHtml(_)));

    // Expect no further message
    let message = receiver.recv_timeout(Duration::from_millis(300));
    assert_matches!(message, Err(TimeoutError));
}

#[test]
fn test_update_loop_detects_file_creation() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file.md");

    fs::write(&path, "# Test").unwrap();
    let renderer = Renderer::new(path.clone());

    let (sender, receiver) = mpsc::channel();
    init_update_loop(renderer, sender);
    // Wait for the watcher thread to get ready
    std::thread::sleep(Duration::from_millis(10));

    fs::remove_file(&path).unwrap();
    fs::write(&path, "# Changed").unwrap();

    // Expect LoadHtml message
    let message = receiver.recv_timeout(Duration::from_millis(300));
    assert_matches!(message, Ok(ui::Event::LoadHtml(_)));

    // Expect no further message
    let message = receiver.recv_timeout(Duration::from_millis(300));
    assert_matches!(message, Err(TimeoutError));
}

#[test]
fn test_update_loop_ignores_unrelated_files() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file.md");
    let other_path = tempdir.path().join("other.md");

    // Create both files
    fs::write(&path, "# Test").unwrap();
    fs::write(&other_path, "# Other").unwrap();

    let renderer = Renderer::new(path.clone());

    let (sender, receiver) = mpsc::channel();
    init_update_loop(renderer, sender);
    // Wait for the watcher thread to get ready
    std::thread::sleep(Duration::from_millis(10));

    // Update only the "other" file
    fs::write(&other_path, "# Updated").unwrap();

    // No message
    let message = receiver.recv_timeout(Duration::from_millis(300));
    assert_matches!(message, Err(TimeoutError));
}
