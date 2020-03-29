use std::fs;
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
    Assets::init().unwrap();
}

#[test]
fn test_multiple_calls_to_delete_work() {
    let mut assets = Assets::init().unwrap();

    assets.delete();
    assets.delete();
}

#[test]
fn test_building_a_file_with_assets_includes_the_given_html() {
    let assets = Assets::init().unwrap();
    let html = "<h1>Example</h1>";

    let path = assets.build(html, &PageState::default()).unwrap();

    assert!(fs::read_to_string(&path).unwrap().contains(html));
    assert!(fs::read_to_string(&path).unwrap().contains("main.js"));
    assert!(fs::read_to_string(&path).unwrap().contains("main.css"));
}

#[test]
fn test_building_a_file_with_assets_includes_main_static_files() {
    let assets = Assets::init().unwrap();
    let path = assets.build("", &PageState::default()).unwrap();

    assert!(fs::read_to_string(&path).unwrap().contains("main.js"));
    assert!(fs::read_to_string(&path).unwrap().contains("main.css"));
}

#[test]
fn test_building_a_file_with_assets_includes_scroll_position_as_the_title() {
    let assets = Assets::init().unwrap();
    let page_state = PageState { scroll_top: 100.5, ..PageState::default() };
    let path = assets.build("", &page_state).unwrap();

    // Yes, it's included as the title, it's kind of dumb, but incredibly easy compared to the
    // alternative.
    assert_contains!(
        fs::read_to_string(&path).unwrap(),
        r#"<title>{"scroll_top":100.5,"image_heights":{}}</title>"#
    );
}
