use std::error::Error;
use std::io;
use std::path::Path;

use gdk::enums::key;
use gtk::prelude::*;
use gtk::{Window, WindowType, HeaderBar};
use log::{debug, warn};
use webkit2gtk::{WebContext, WebView, WebViewExt};

use crate::assets::Assets;

pub enum Event {
    LoadHtml(String),
    Reload,
}

pub fn init_render_loop(mut app: App, gui_receiver: glib::Receiver<Event>) {
    gui_receiver.attach(None, move |event| {
        match event {
            Event::LoadHtml(html) => {
                app.load_html(&html).
                    unwrap_or_else(|e| warn!("Couldn't update HTML: {}", e))
            },
            Event::Reload => app.reload(),
        }
        glib::Continue(true)
    });
}

#[derive(Clone)]
pub struct App {
    window: Window,
    header_bar: HeaderBar,
    webview: WebView,
    assets: Assets,
}

impl App {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(1024, 768);

        let header_bar = HeaderBar::new();
        header_bar.set_title("Quickmd");
        header_bar.set_show_close_button(true);

        let web_context = WebContext::get_default().ok_or_else(|| format!("Couldn't initialize GTK WebContext"))?;
        let webview = WebView::new_with_context(&web_context);

        window.set_titlebar(&header_bar);
        window.add(&webview);

        let assets = Assets::init()?;

        Ok(App { window, header_bar, webview, assets })
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

    pub fn load_html(&mut self, html: &str) -> Result<(), io::Error> {
        let scroll_top = self.webview.get_title().
            and_then(|t| t.parse::<f64>().ok()).
            unwrap_or(0.0);

        let output_path = self.assets.build(html, scroll_top)?;

        debug!("Loading HTML:");
        debug!(" > output_path = {}", output_path.display());

        self.webview.load_uri(&format!("file://{}", output_path.display()));
        Ok(())
    }

    pub fn reload(&self) {
        self.webview.reload();
    }

    pub fn run(&self) {
        self.window.show_all();
        gtk::main();
    }
}
