use gtk::prelude::*;
use gtk::{Window, WindowType};
use webkit2gtk::{WebContext, WebView, WebViewExt};
use std::env;
use std::path::Path;

fn main() {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        return;
    }

    let argument = match env::args().nth(1) {
        Some(f) => f,
        None => {
            eprintln!("USAGE: quickmd <file.md>");
            return;
        },
    };
    let path = match Path::new(&argument).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error reading {}: {}", argument, e);
            return
        },
    };

    let window = Window::new(WindowType::Toplevel);
    window.set_title(&format!("Quickmd: {}", path.display()));
    //window.set_default_size(350, 70);

    // Create a the WebView for the preview pane.
    let context = WebContext::get_default().unwrap();
    let preview = WebView::new_with_context(&context);
    preview.load_uri(&format!("file://{}", path.display()));

    window.add(&preview);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
