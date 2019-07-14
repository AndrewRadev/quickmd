use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

use dirs::home_dir;
use gdk::enums::key;
use gtk::prelude::*;
use gtk::{Window, WindowType, HeaderBar};
use log::debug;
use tempfile::{tempdir, TempDir};
use webkit2gtk::{WebContext, WebView, WebViewExt};

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
        let src_root = option_env!("PWD").unwrap_or("");
        let home_path = home_dir().
            map(|p| p.display().to_string()).
            unwrap_or(String::new());
        let scroll_top = self.webview.get_title().
            and_then(|t| t.parse::<f64>().ok()).
            unwrap_or(0.0);

        debug!("Building HTML:");
        debug!(" > src_root   = {}", src_root);
        debug!(" > home_path  = {}", home_path);
        debug!(" > scroll_top = {}", scroll_top);

        let page = format! {
            include_str!("../res/layout.html"),
            src_root=src_root,
            home_path=home_path,
            body=html,
            scroll_top=scroll_top,
        };

        let html_path = self.temp_dir.path().join("output.html");
        {
            let mut f = File::create(&html_path).unwrap();
            f.write(page.as_bytes()).unwrap();
            f.flush().unwrap();
        }

        debug!("Loading HTML:");
        debug!(" > html_path = {}", html_path.display());

        self.webview.load_uri(&format!("file://{}", html_path.display()));
    }

    pub fn reload(&self) {
        self.webview.reload();
    }

    pub fn run(&self) {
        self.window.show_all();
        gtk::main();
    }
}
