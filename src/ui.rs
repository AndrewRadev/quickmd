use std::fs;
use std::path::Path;
use std::rc::Rc;

use dirs::home_dir;
use gdk::enums::key;
use gtk::prelude::*;
use gtk::{Window, WindowType, HeaderBar};
use log::{debug, warn};
use tempfile::{tempdir, TempDir};
use webkit2gtk::{WebContext, WebView, WebViewExt};

const MAIN_JS:    &'static str = include_str!("../res/js/main.js");
const MAIN_CSS:   &'static str = include_str!("../res/style/main.css");
const GITHUB_CSS: &'static str = include_str!("../res/style/github.css");

pub enum Event {
    LoadHtml(String),
    Reload,
}

#[derive(Clone)]
pub struct App {
    window: Window,
    header_bar: HeaderBar,
    webview: WebView,
    temp_dir: Rc<TempDir>,
}

impl App {
    pub fn init() -> Self {
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(1024, 768);

        let header_bar = HeaderBar::new();
        header_bar.set_title("Quickmd");
        header_bar.set_show_close_button(true);

        let web_context = WebContext::get_default().unwrap();
        let webview = WebView::new_with_context(&web_context);

        window.set_titlebar(&header_bar);
        window.add(&webview);

        App {
            window, header_bar, webview,
            temp_dir: Rc::new(tempdir().unwrap()),
        }
    }

    pub fn set_filename(&self, filename: &Path) {
        self.header_bar.set_title(filename.to_str());
    }

    pub fn connect_events(&self) {
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

    pub fn load_html(&mut self, html: &str) {
        let home_path = home_dir().
            map(|p| p.display().to_string()).
            unwrap_or(String::new());
        let scroll_top = self.webview.get_title().
            and_then(|t| t.parse::<f64>().ok()).
            unwrap_or(0.0);

        debug!("Building HTML:");
        debug!(" > home_path  = {}", home_path);
        debug!(" > scroll_top = {}", scroll_top);

        fs::write(self.temp_dir.path().join("main.js"), MAIN_JS).unwrap_or_else(|e| warn!("{}", e));
        fs::write(self.temp_dir.path().join("main.css"), MAIN_CSS).unwrap_or_else(|e| warn!("{}", e));
        fs::write(self.temp_dir.path().join("github.css"), GITHUB_CSS).unwrap_or_else(|e| warn!("{}", e));

        let page = format! {
            include_str!("../res/layout.html"),
            home_path=home_path,
            body=html,
            scroll_top=scroll_top,
        };

        let output_path = self.temp_dir.path().join("output.html");
        fs::write(&output_path, page.as_bytes()).unwrap();

        debug!("Loading HTML:");
        debug!(" > output_path = {}", output_path.display());

        self.webview.load_uri(&format!("file://{}", output_path.display()));
    }

    pub fn reload(&self) {
        self.webview.reload();
    }

    pub fn run(&self) {
        self.window.show_all();
        gtk::main();
    }
}
