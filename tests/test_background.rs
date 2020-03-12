use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;

use tempfile::TempDir;

use quickmd::ui;
use quickmd::markdown::Renderer;
use quickmd::background::init_update_loop;

struct WorkingDir {
    original_dir: PathBuf,
    _tempdir: TempDir,
}

impl WorkingDir {
    fn temp() -> Self {
        let current_dir = env::current_dir().unwrap();
        let tempdir = TempDir::new().unwrap();
        env::set_current_dir(tempdir.path()).unwrap();

        WorkingDir { original_dir: current_dir, _tempdir: tempdir }
    }
}

impl Drop for WorkingDir {
    fn drop(&mut self) {
        env::set_current_dir(&self.original_dir).unwrap();
    }
}

#[test]
fn test_update_loop_detects_file_updates() {
    let _working_dir = WorkingDir::temp();

    fs::write("file.md", "# Test").unwrap();
    let path = PathBuf::from("file.md");
    let renderer = Renderer::new(path);

    let (sender, receiver) = mpsc::channel();
    init_update_loop(renderer, sender);

    // Initial render
    let message = receiver.recv().unwrap();
    assert!(matches!(message, ui::Event::LoadHtml(_)));

    fs::write("file.md", "# Changed").unwrap();

    // Updated render
    let message = receiver.recv().unwrap();
    assert!(matches!(message, ui::Event::LoadHtml(_)));

    // TODO Handle no-render case with a timeout.
}
