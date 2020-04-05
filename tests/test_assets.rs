use std::fs;
use claim::assert_matches;
use quickmd::assets::{Assets, PageState};

macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {
        if $haystack.contains($needle) {
            assert!(true);
        } else {
            assert!(false, "\n\nExpected:\n\n{}\nto contain:\n\n{}\n\n", $haystack, $needle)
        }
    }
}

#[test]
fn test_initialization_works() {
    Assets::init(None).unwrap();
}

#[test]
fn test_multiple_cleanups_work() {
    let mut assets = Assets::init(None).unwrap();

    assets.clean_up();
    assets.clean_up();
}

#[test]
fn test_building_a_file_with_assets_includes_the_given_html() {
    let assets = Assets::init(None).unwrap();
    let html = "<h1>Example</h1>";

    let path = assets.build(html, &PageState::default()).unwrap();

    assert!(fs::read_to_string(&path).unwrap().contains(html));
    assert!(fs::read_to_string(&path).unwrap().contains("main.js"));
    assert!(fs::read_to_string(&path).unwrap().contains("main.css"));
}

#[test]
fn test_building_a_file_with_assets_includes_main_static_files() {
    let assets = Assets::init(None).unwrap();
    let path = assets.build("", &PageState::default()).unwrap();

    assert!(fs::read_to_string(&path).unwrap().contains("main.js"));
    assert!(fs::read_to_string(&path).unwrap().contains("main.css"));
}

#[test]
fn test_building_a_file_with_assets_includes_scroll_position_as_the_title() {
    let assets = Assets::init(None).unwrap();
    let page_state = PageState { scroll_top: 100.5, ..PageState::default() };
    let path = assets.build("", &page_state).unwrap();

    // Yes, it's included as the title, it's kind of dumb, but incredibly easy compared to the
    // alternative.
    assert_contains!(
        fs::read_to_string(&path).unwrap(),
        r#"<title>{"scroll_top":100.5,"image_widths":{},"image_heights":{}}</title>"#
    );
}

#[test]
fn test_output_to_a_given_directory() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().to_path_buf();
    let mut assets = Assets::init(Some(path.clone())).unwrap();

    assert!(!path.join("index.html").exists());
    assets.build("", &PageState::default()).unwrap();
    assert!(path.join("index.html").exists());

    // Clearing asset tempdir should not remove explicitly given directory
    assets.clean_up();
    assert!(path.join("index.html").exists());

    // Dropping assets should also not remove directory
    drop(assets);
    assert!(path.join("index.html").exists());
}

#[test]
fn test_will_create_output_dir_if_it_doesnt_exist() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("nested").join("output");
    let assets = Assets::init(Some(path.clone())).unwrap();

    assert!(!path.join("index.html").exists());
    assets.build("", &PageState::default()).unwrap();
    assert!(path.join("index.html").exists());
}

#[test]
fn test_will_fail_if_output_dir_is_not_a_dir() {
    let tempfile = tempfile::NamedTempFile::new().unwrap();
    let path = tempfile.path().to_path_buf();

    assert_matches!(Assets::init(Some(path)), Err(_));
}
