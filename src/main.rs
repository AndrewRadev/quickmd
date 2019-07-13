use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use std::fs::File;
use std::io::Write;

use dirs::home_dir;
use gdk::enums::key;
use gtk::prelude::*;
use gtk::{Window, WindowType};
use notify::{Watcher, RecursiveMode, watcher};
use pulldown_cmark::{Parser, html};
use webkit2gtk::{WebContext, WebView, WebViewExt};
use tempfile::{tempdir, TempDir};

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

struct UserInterface {
    window: Window,
    webview: WebView,
    temp_dir: TempDir,
}

impl Clone for UserInterface {
    fn clone(&self) -> Self {
        UserInterface {
            window: self.window.clone(),
            webview: self.webview.clone(),
            temp_dir: tempdir().unwrap(),
        }
    }
}

impl UserInterface {
    fn init() -> Self {
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(1024, 768);

        let web_context = WebContext::get_default().unwrap();
        let webview = WebView::new_with_context(&web_context);

        window.add(&webview);

        UserInterface {
            window, webview,
            temp_dir: tempdir().unwrap(),
        }
    }

    fn set_filename(&self, filename: &Path) {
        self.window.set_title(&format!("Quickmd: {}", filename.display()));
    }

    fn connect_events(&self) {
        // Each key press will invoke this function.
        self.window.connect_key_press_event(move |_window, gdk| {
            match gdk.get_keyval() {
                key::Escape => gtk::main_quit(),
                _ => (),
            }
            Inhibit(false)
        });

        self.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
    }

    fn load_html(&mut self, html: &str) {
        let src_root = option_env!("PWD").unwrap_or("");
        let home_path = home_dir().
            map(|p| p.display().to_string()).
            unwrap_or(String::new());
        let scroll_top = self.webview.get_title().
            and_then(|t| t.parse::<f64>().ok()).
            unwrap_or(0.0);

        let page = format! {
            include_str!("../resources/template.html"),
            src_root=src_root,
            home_path=home_path,
            body=html,
            scroll_top=scroll_top,
        };

        let html_path = self.temp_dir.path().join("content.html");
        {
            let mut f = File::create(&html_path).unwrap();
            f.write(page.as_bytes()).unwrap();
            f.flush().unwrap();
        }

        self.webview.load_uri(&format!("file://{}", html_path.display()));
    }

    fn reload(&self) {
        self.webview.reload();
    }

    fn run(&self) {
        self.window.show_all();
        gtk::main();
    }
}

enum UiEvent {
    LoadHtml(String),
    Reload,
}

fn init_watch_loop(content: Content, gui_sender: glib::Sender<UiEvent>) {
    thread::spawn(move || {
        let (watcher_sender, watcher_receiver) = mpsc::channel();
        let mut watcher = watcher(watcher_sender, Duration::from_millis(200)).unwrap();
        watcher.watch(&content.md_path, RecursiveMode::NonRecursive).unwrap();

        if let Some(home) = home_dir() {
            let _ = watcher.watch(home.join(".quickmd.css"), RecursiveMode::NonRecursive);
            let _ = watcher.watch(home.join(".config/quickmd.css"), RecursiveMode::NonRecursive);
        }

        loop {
            use notify::DebouncedEvent::*;

            match watcher_receiver.recv() {
                Ok(Write(file)) => {
                    if file == content.md_path {
                        match content.render() {
                            Ok(html) => {
                                let _ = gui_sender.send(UiEvent::LoadHtml(html));
                            },
                            Err(e) => {
                                eprintln! {
                                    "Error rendering markdown ({}): {:?}",
                                    content.md_path.display(), e
                                };
                            }
                        }
                    } else {
                        let _ = gui_sender.send(UiEvent::Reload);
                    }
                },
                Ok(_) => {
                    // TODO consider "verbose mode" with output
                }
                Err(e) => eprintln!("Error watching file for changes: {:?}", e),
            }
        }
    });
}

fn init_ui_render_loop(mut ui: UserInterface, gui_receiver: glib::Receiver<UiEvent>) {
    gui_receiver.attach(None, move |event| {
        match event {
            UiEvent::LoadHtml(html) => ui.load_html(&html),
            UiEvent::Reload => ui.reload(),
        }
        glib::Continue(true)
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    better_panic::install();

    gtk::init()?;

    let input = env::args().nth(1).ok_or_else(|| {
        format!("USAGE: quickmd <file.md>")
    })?;

    let md_path = Path::new(&input).canonicalize()?;
    let content = Content::new(md_path);

    let mut ui = UserInterface::init();
    let html = content.render().map_err(|e| {
        format!("Couldn't parse markdown from file {}: {}", content.md_path.display(), e)
    })?;

    ui.set_filename(&content.md_path);
    ui.connect_events();
    ui.load_html(&html);

    let (gui_sender, gui_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    init_watch_loop(content.clone(), gui_sender);
    init_ui_render_loop(ui.clone(), gui_receiver);

    ui.run();
    Ok(())
}
