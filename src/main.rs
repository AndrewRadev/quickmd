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

#[derive(Clone)]
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

#[derive(Clone)]
struct UserInterface {
    window: Window,
    webview: WebView,
}

impl UserInterface {
    fn init() -> Self {
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(1024, 768);

        let context = WebContext::get_default().unwrap();
        let webview = WebView::new_with_context(&context);

        window.add(&webview);

        UserInterface { window, webview }
    }

    fn set_filename(&self, filename: &Path) {
        self.window.set_title(&format!("Quickmd: {}", filename.display()));
    }

    fn load_html(&mut self, html: &str) {
        self.webview.load_html(html, None);
    }

    fn run(&self) {
        self.window.show_all();

        self.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        gtk::main();
    }
}

fn init_watch_loop(content: Content, gui_sender: glib::Sender<String>) {
    thread::spawn(move || {
        let (watcher_sender, watcher_receiver) = mpsc::channel();
        let mut watcher = watcher(watcher_sender, Duration::from_millis(200)).unwrap();
        watcher.watch(&content.md_path, RecursiveMode::NonRecursive).unwrap();

        loop {
            match watcher_receiver.recv() {
                Ok(_) => {
                    match content.render() {
                        Ok(html) => {
                            let _ = gui_sender.send(html);
                        },
                        Err(e) => {
                            eprintln! {
                                "Error rendering markdown ({}): {:?}",
                                content.md_path.display(), e
                            };
                        }
                    }
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
}

fn init_ui_render_loop(mut ui: UserInterface, gui_receiver: glib::Receiver<String>) {
    gui_receiver.attach(None, move |html| {
        ui.load_html(&html);
        glib::Continue(true)
    });
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

    let mut ui = UserInterface::init();
    let html = content.render().unwrap_or_else(|e| {
        eprintln!("Couldn't parse markdown from file {}: {}", content.md_path.display(), e);
        exit(1);
    });

    ui.set_filename(&content.md_path);
    ui.load_html(&html);

    let (gui_sender, gui_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    init_watch_loop(content.clone(), gui_sender);
    init_ui_render_loop(ui.clone(), gui_receiver);

    ui.run();
}
