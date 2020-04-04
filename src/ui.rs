//! The GTK user interface.

use anyhow::anyhow;
use gdk::enums::key;
use gtk::prelude::*;
use gtk::{Window, WindowType};
use log::{debug, warn};
use pathbuftools::PathBufTools;
use webkit2gtk::{WebContext, WebView, WebViewExt};

use crate::assets::{Assets, PageState};
use crate::input::InputFile;

/// Events that trigger UI changes.
///
#[derive(Debug)]
pub enum Event {
    /// Load the given HTML string into the webview.
    LoadHtml(String),
    /// Refresh the webview.
    Reload,
}

/// The container for all the GTK widgets of the app -- window, webview, etc.
/// Reference-counted, so should be cheap to clone.
///
#[derive(Clone)]
pub struct App {
    window: Window,
    webview: WebView,
    assets: Assets,
}

impl App {
    /// Construct a new app.
    ///
    /// The `input_file` parameter is used as the window title and for other actions on the file.
    /// Initialization could fail due to `WebContext` or `Assets` failures.
    ///
    pub fn init(input_file: InputFile) -> anyhow::Result<Self> {
        let window = Window::new(WindowType::Toplevel);
        window.set_default_size(1024, 768);

        let title = match input_file {
            InputFile::Filesystem(p) => format!("{} - Quickmd", p.short_path().display()),
            InputFile::Stdin(_)      => format!("Quickmd"),
        };
        window.set_title(&title);

        let web_context = WebContext::get_default().
            ok_or_else(|| anyhow!("Couldn't initialize GTK WebContext"))?;
        let webview = WebView::new_with_context(&web_context);
        window.add(&webview);

        let assets = Assets::init()?;

        Ok(App { window, webview, assets })
    }

    /// Start listening to events from the `ui_receiver` and trigger the relevant methods on the
    /// `App`. Doesn't block.
    ///
    pub fn init_render_loop(&self, ui_receiver: glib::Receiver<Event>) {
        let mut app_clone = self.clone();

        ui_receiver.attach(None, move |event| {
            match event {
                Event::LoadHtml(html) => {
                    app_clone.load_html(&html).
                        unwrap_or_else(|e| warn!("Couldn't update HTML: {}", e))
                },
                Event::Reload => app_clone.reload(),
            }
            glib::Continue(true)
        });
    }

    /// Actually start the UI, blocking the main thread.
    ///
    pub fn run(&self) {
        self.connect_events();
        self.window.show_all();
        gtk::main();
    }

    fn load_html(&mut self, html: &str) -> anyhow::Result<()> {
        let page_state = match self.webview.get_title() {
            Some(t) => {
                serde_json::from_str(t.as_str()).unwrap_or_else(|e| {
                    warn!("Failed to get page state from {}: {:?}", t, e);
                    PageState::default()
                })
            },
            None => PageState::default(),
        };
        let output_path = self.assets.build(html, &page_state)?;

        debug!("Loading HTML:");
        debug!(" > output_path = {}", output_path.display());

        self.webview.load_uri(&format!("file://{}", output_path.display()));
        Ok(())
    }

    fn reload(&self) {
        self.webview.reload();
    }

    fn connect_events(&self) {
        use std::cell::RefCell;
        let self_clone = RefCell::new(Some(self.clone()));

        // Each key press will invoke this function.
        self.window.connect_key_press_event(move |_window, gdk| {
            if let key::Escape = gdk.get_keyval() {
                self_clone.borrow_mut().take().unwrap().assets.delete();
                gtk::main_quit()
            }
            Inhibit(false)
        });

        self.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
    }
}
