//! Management of external assets like Javascript and CSS.
//!
//! The files are stored into the binary as strings and written to the filesystem when the
//! application runs. For the HTML file, this allows the webview to load a URL instead of a string
//! body, which makes reloading smoother (update the file, refresh).
//!
//! For the other assets, it means the HTML can refer to local files instead of embedding the
//! contents as `<script>` and `<style>` tags, making the output easier to read and debug.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::anyhow;
use dirs::home_dir;
use log::{debug, warn};
use serde::{Serialize, Deserialize};
use tempfile::{tempdir, TempDir};

const MAIN_JS:    &str = include_str!("../res/js/main.js");
const MAIN_CSS:   &str = include_str!("../res/style/main.css");
const GITHUB_CSS: &str = include_str!("../res/style/github.css");

/// The client-side state of the page as the user's interacted with it. Currently, includes the
/// scroll position and the heights of images on the page (Note: not yet implemented), so that
/// reloading doesn't change the viewport.
///
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PageState {
    /// Scroll position of the page.
    pub scroll_top: f64,

    /// A cache of all the widths of images in the page, keyed by src URL.
    pub image_widths: HashMap<String, f64>,

    /// A cache of all the heights of images in the page, keyed by src URL.
    pub image_heights: HashMap<String, f64>,
}

/// A container for static assets.
///
/// Has a temporary directory where it builds everything. Internally reference-counted, so clones
/// share the same storage.
///
#[derive(Clone)]
pub struct Assets {
    temp_dir: Option<Rc<TempDir>>,
}

impl Assets {
    /// Create a new instance. It should never be necessary to create more than one, but it's
    /// possible.
    ///
    pub fn init() -> Result<Self, io::Error> {
        let temp_dir = tempdir()?;

        fs::write(temp_dir.path().join("main.js"), MAIN_JS).
            unwrap_or_else(|e| warn!("{}", e));
        fs::write(temp_dir.path().join("main.css"), MAIN_CSS).
            unwrap_or_else(|e| warn!("{}", e));
        fs::write(temp_dir.path().join("github.css"), GITHUB_CSS).
            unwrap_or_else(|e| warn!("{}", e));

        Ok(Assets { temp_dir: Some(Rc::new(temp_dir)) })
    }

    /// Given an HTML fragment, wrap it up in whatever is necessary to turn it into a proper
    /// preview page and write it to a file.
    ///
    /// Input:
    ///
    /// - `html`:       The HTML fragment to write to a file
    /// - `scroll_top`: A scroll position to embed in the document, so it can read it via javascript
    ///                 and reposition itself.
    ///
    /// Returns the path to the generated HTML file, or an error.
    ///
    pub fn build(&self, html: &str, page_state: &PageState) -> anyhow::Result<PathBuf> {
        let temp_dir = self.temp_dir.clone().
            ok_or_else(|| anyhow!("TempDir deleted, there might be a synchronization error"))?;

        let home_path = home_dir().
            map(|p| p.display().to_string()).
            unwrap_or_else(String::new);

        let json_state = serde_json::to_string(page_state).
            unwrap_or_else(|e| {
                warn!("Couldn't build JSON state from {:?}: {:?}", page_state, e);
                String::from("{}")
            });

        debug!("Building HTML:");
        debug!(" > home_path  = {}", home_path);
        debug!(" > page_state = {:?}", json_state);

        let page = format! {
            include_str!("../res/layout.html"),
            home_path=home_path,
            body=html,
            page_state=json_state,
        };

        let output_path = temp_dir.path().join("output.html");
        fs::write(&output_path, page.as_bytes())?;

        Ok(output_path)
    }

    /// Delete all the storage for the structure. This should happen automatically on drop, but a
    /// GTK-level exit doesn't seem to unroll the stack, so we may need to delete things
    /// explicitly.
    ///
    /// If deletion fails, we quietly print a warning. Multiple (successful or failed) deletions
    /// are a noop.
    ///
    pub fn delete(&mut self) {
        if let Some(temp_dir) = self.temp_dir.take() {
            let path = temp_dir.path();
            fs::remove_dir_all(path).unwrap_or_else(|_| {
                warn!("Couldn't delete temporary dir: {}", path.display());
            });
        }
    }
}
