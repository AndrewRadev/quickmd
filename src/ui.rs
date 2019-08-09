use std::convert::AsRef;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;

use gdk::enums::key;
use gtk::prelude::*;
use gtk::{ApplicationWindow, HeaderBar};
use log::{debug, warn};
use webkit2gtk::{WebContext, WebView, WebViewExt};

use crate::assets::Assets;

pub enum Event {
    LoadHtml(String),
    Reload,
}

pub fn init_render_loop(mut window: MainWindow, gui_receiver: glib::Receiver<Event>) {
    gui_receiver.attach(None, move |event| {
        match event {
            Event::LoadHtml(html) => {
                window.load_html(&html).
                    unwrap_or_else(|e| warn!("Couldn't update HTML: {}", e))
            },
            Event::Reload => window.reload(),
        }
        glib::Continue(true)
    });
}

#[derive(Clone)]
pub struct App {
    pub gtk_app: Rc<gtk::Application>,
}

impl App {
    pub fn new(gtk_app: gtk::Application) -> Self {
        let gtk_app = Rc::new(gtk_app);
        App { gtk_app }
    }
}

impl AsRef<gtk::Application> for App {
    fn as_ref(&self) -> &gtk::Application {
        &self.gtk_app
    }
}

#[derive(Clone)]
pub struct MainWindow {
    gtk_window: ApplicationWindow,
    header_bar: HeaderBar,
    webview: WebView,
    assets: Assets,
}

impl MainWindow {
    pub fn new(app: App) -> Result<Self, Box<dyn Error>> {
        let gtk_window = ApplicationWindow::new(app.as_ref());
        gtk_window.set_position(gtk::WindowPosition::Center);
        gtk_window.set_default_size(1024, 768);

        let header_bar = HeaderBar::new();
        header_bar.set_title("Quickmd");
        header_bar.set_show_close_button(true);

        let web_context = WebContext::get_default().
            ok_or_else(|| format!("Couldn't initialize GTK WebContext"))?;
        let webview = WebView::new_with_context(&web_context);

        gtk_window.set_titlebar(&header_bar);
        gtk_window.add(&webview);

        let assets = Assets::init()?;

        Ok(MainWindow { gtk_window, header_bar, webview, assets })
    }

    pub fn set_filename(&self, filename: &Path) {
        self.header_bar.set_title(filename.to_str());
    }

    pub fn connect_events(&self) {
        // Each key press will invoke this function.
        use std::cell::RefCell;
        let self_clone = RefCell::new(Some(self.clone()));
        self.gtk_window.connect_key_press_event(move |_window, gdk| {
            match gdk.get_keyval() {
                key::Escape => self_clone.borrow_mut().take().unwrap().close(),
                _ => (),
            }
            Inhibit(false)
        });
    }

    pub fn load_html(&mut self, html: &str) -> Result<(), Box<dyn Error>> {
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

    pub fn show(&self) {
        self.gtk_window.show_all();
    }

    pub fn close(&mut self) {
        self.assets.delete();
        self.gtk_window.close();
    }
}
