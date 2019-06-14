use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

use gtk::prelude::*;
use gtk::{Window, WindowType};
use webkit2gtk::{WebContext, WebView, WebViewExt};
use pulldown_cmark::{Parser, html};

fn main() {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        exit(1);
    }

    let argument = match env::args().nth(1) {
        Some(f) => f,
        None => {
            eprintln!("USAGE: quickmd <file.md>");
            exit(1);
        },
    };
    let path = match Path::new(&argument).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error reading {}: {}", argument, e);
            exit(1);
        },
    };

    let window = Window::new(WindowType::Toplevel);
    window.set_title(&format!("Quickmd: {}", path.display()));
    //window.set_default_size(350, 70);

    let markdown = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Couldn't read markdown file: {}", path.display());
            exit(1);
        },
    };

    // Parse markdown
    let parser = Parser::new(&markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Create a the WebView for the preview pane.
    let context = WebContext::get_default().unwrap();
    let preview = WebView::new_with_context(&context);
    preview.load_html(&html_output, None);

    window.add(&preview);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
