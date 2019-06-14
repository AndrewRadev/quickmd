use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

use gtk::prelude::*;
use gtk::{Window, WindowType};
use webkit2gtk::{WebContext, WebView, WebViewExt};
use pulldown_cmark::{Parser, html};

struct Content {
    md_path: PathBuf,
    html_output: String,
}

impl Content {
    fn from_path(input_path: &str) -> Result<Self, Box<Error>> {
        let md_path = Path::new(input_path).canonicalize()?;
        let markdown = fs::read_to_string(&md_path)?;

        let parser = Parser::new(&markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(Content { md_path, html_output })
    }
}

fn main() {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        exit(1);
    }

    let input_path = match env::args().nth(1) {
        Some(f) => f,
        None => {
            eprintln!("USAGE: quickmd <file.md>");
            exit(1);
        },
    };

    let content = match Content::from_path(&input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    let window = Window::new(WindowType::Toplevel);
    window.set_title(&format!("Quickmd: {}", content.md_path.display()));
    //window.set_default_size(350, 70);

    // Create a the WebView for the preview pane.
    let context = WebContext::get_default().unwrap();
    let preview = WebView::new_with_context(&context);
    preview.load_html(&content.html_output, None);

    window.add(&preview);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
