use std::time::Instant;

use anyhow::anyhow;
use gio::Cancellable;
use gtk::prelude::*;
use log::{debug, warn};
use webkit2gtk::traits::WebViewExt;
use webkit2gtk::{WebContext, WebView};

use crate::assets::PageState;
use crate::input::Config;

/// A thin layer on top of `webkit2gtk::WebView` to put helper methods into.
///
#[derive(Clone)]
pub struct Browser {
    webview: WebView,
    config: Config,
}

impl Browser {
    /// Construct a new instance with the provided `Config`.
    ///
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let web_context = WebContext::default().
            ok_or_else(|| anyhow!("Couldn't initialize GTK WebContext"))?;
        let webview = WebView::with_context(&web_context);
        webview.set_zoom_level(config.zoom);

        Ok(Browser { webview, config })
    }

    /// Add this browser instance's webview to the given GTK Window.
    ///
    pub fn attach_to(&self, window: &gtk::Window) {
        window.add(&self.webview);
    }

    /// Delegates to `webkit2gtk::WebView`
    pub fn load_uri(&self, uri: &str) {
        self.webview.load_uri(uri);
    }

    /// Delegates to `webkit2gtk::WebView`
    pub fn reload(&self) {
        self.webview.reload();
    }

    /// Increase zoom level by ~10%
    ///
    pub fn zoom_in(&self) {
        let zoom_level = self.webview.zoom_level();
        self.webview.set_zoom_level(zoom_level + 0.1);
        debug!("Zoom level set to: {}", zoom_level);
    }

    /// Decrease zoom level by ~10%, down till 20% or so.
    ///
    pub fn zoom_out(&self) {
        let zoom_level = self.webview.zoom_level();

        if zoom_level > 0.2 {
            self.webview.set_zoom_level(zoom_level - 0.1);
            debug!("Zoom level set to: {}", zoom_level);
        }
    }

    /// Reset to the base zoom level defined in the config (which defaults to 100%).
    ///
    pub fn zoom_reset(&self) {
        self.webview.set_zoom_level(self.config.zoom);
        debug!("Zoom level set to: {}", self.config.zoom);
    }

    /// Get the deserialized `PageState` from the current contents of the webview. This is later
    /// rendered unchanged into the HTML content.
    ///
    pub fn get_page_state(&self) -> PageState {
        match self.webview.title() {
            Some(t) => {
                serde_json::from_str(t.as_str()).unwrap_or_else(|e| {
                    warn!("Failed to get page state from {}: {:?}", t, e);
                    PageState::default()
                })
            },
            None => PageState::default(),
        }
    }

    /// Execute some (async) javascript code in the webview, without checking the result other than
    /// printing a warning if it errors out.
    ///
    pub fn execute_js(&self, js_code: &'static str) {
        let now = Instant::now();

        self.webview.run_javascript(js_code, None::<&Cancellable>, move |result| {
            if let Err(e) = result {
                warn!("Javascript execution error: {}", e);
            } else {
                debug!("Javascript executed in {}ms:\n> {}", now.elapsed().as_millis(), js_code);
            }
        });
    }
}
