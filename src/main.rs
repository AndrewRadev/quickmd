use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{Window, WindowType};
use notify::{Watcher, RecursiveMode, watcher};
use pulldown_cmark::{Parser, html};
use webkit2gtk::{WebContext, WebView, WebViewExt};

struct Content {
    md_path: PathBuf,
}

impl Content {
    fn new(md_path: PathBuf) -> Self {
        Content { md_path }
    }

    fn render(&self) -> Result<String, io::Error> {
        let markdown = fs::read_to_string(&self.md_path)?;

        let parser = Parser::new(&markdown);
        let mut output = String::new();
        html::push_html(&mut output, parser);
        Ok(output)
    }
}

fn main() {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        exit(1);
    }

    let input = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("USAGE: quickmd <file.md>");
        exit(1);
    });
    let md_path = Path::new(&input).canonicalize().unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1);
    });
    let content = Content::new(md_path);

    let window = Window::new(WindowType::Toplevel);
    window.set_title(&format!("Quickmd: {}", content.md_path.display()));
    //window.set_default_size(350, 70);

    // Create a the WebView for the preview pane.
    let context = WebContext::get_default().unwrap();
    let preview = WebView::new_with_context(&context);
    let html = content.render().unwrap_or_else(|e| {
        eprintln!("Couldn't parse markdown from file {}: {}", content.md_path.display(), e);
        exit(1);
    });
    preview.load_html(&html, None);

    window.add(&preview);
    window.show_all();

    let (gui_sender, gui_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || {
        let (watcher_sender, watcher_receiver) = mpsc::channel();
        let mut watcher = watcher(watcher_sender, Duration::from_millis(200)).unwrap();
        watcher.watch(&content.md_path, RecursiveMode::NonRecursive).unwrap();

        loop {
            match watcher_receiver.recv() {
                Ok(_) => {
                    let _ = gui_sender.send(content.render());
                },
                Err(e) => {
                    eprintln! {
                        "Error watching file for changes ({}): {:?}",
                        content.md_path.display(), e
                    };
                },
            }
        }
    });

    let preview_clone = preview.clone();
    gui_receiver.attach(None, move |message| {
        if let Ok(html) = message {
            preview_clone.load_html(&html, None);
        }
        glib::Continue(true)
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
