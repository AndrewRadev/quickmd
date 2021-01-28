//! Management of external assets like Javascript and CSS.
//!
//! The files are stored into the binary as strings and written to the filesystem when the
//! application runs. For the HTML file, this allows the webview to load a URL instead of a string
//! body, which makes reloading smoother (update the file, refresh).
//!
//! For the other assets, it means the HTML can refer to local files instead of embedding the
//! contents as `<script>` and `<style>` tags, making the output easier to read and debug.

use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::anyhow;
use log::{debug, warn};
use serde::{Serialize, Deserialize};
use tempfile::{tempdir, TempDir};

use crate::input::Config;
use crate::markdown::RenderedContent;

const MAIN_JS:    &str = include_str!("../res/js/main.js");
const MAIN_CSS:   &str = include_str!("../res/style/main.css");
const GITHUB_CSS: &str = include_str!("../res/style/github.css");

/// The version of highlight.js the app uses for code highlighting.
///
/// More details about the tool at https://highlightjs.org/
///
pub const HIGHLIGHT_JS_VERSION: &str = "9.18.1";

/// The client-side state of the page as the user's interacted with it. Currently, includes the
/// scroll position and the dimensions of images on the page, so that reloading doesn't change the
/// viewport.
///
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PageState {
    /// Scroll position of the page.
    pub scroll_top: f64,

    /// A cache of all the widths of images in the page, keyed by their URLs.
    pub image_widths: HashMap<String, f64>,

    /// A cache of all the heights of images in the page, keyed by their URLs.
    pub image_heights: HashMap<String, f64>,
}

/// A container for static assets.
///
/// Builds everything in either the explicitly-given directory, or a temporary one. Internally
/// reference-counted, so clones share the same storage.
///
#[derive(Debug, Clone)]
pub struct Assets {
    real_dir: Option<PathBuf>,
    temp_dir: Option<Rc<TempDir>>,
}

impl Assets {
    /// Create a new instance. It should never be necessary to create more than one, but it's
    /// possible.
    ///
    /// If the optional `output_dir` parameter is not given, the instance will use a temporary
    /// directory.
    ///
    /// If `output_dir` doesn't exist, it will be recursively created.
    ///
    pub fn init(output_dir: Option<PathBuf>) -> Result<Self, io::Error> {
        let assets =
            if let Some(real_dir) = output_dir {
                if !real_dir.is_dir() {
                    fs::create_dir_all(&real_dir)?;
                }

                let real_dir = Some(real_dir.canonicalize()?);
                Assets { real_dir, temp_dir: None }
            } else {
                let temp_dir = tempdir()?;
                let temp_dir = Some(Rc::new(temp_dir));
                Assets { temp_dir, real_dir: None }
            };
        // [Unwrap] We just constructed it, so an output path should exist:
        let output_path = assets.output_path().unwrap();

        fs::write(output_path.join("main.js"), MAIN_JS).
            unwrap_or_else(|e| warn!("{}", e));
        fs::write(output_path.join("main.css"), MAIN_CSS).
            unwrap_or_else(|e| warn!("{}", e));
        fs::write(output_path.join("github.css"), GITHUB_CSS).
            unwrap_or_else(|e| warn!("{}", e));

        Ok(assets)
    }

    /// Given an HTML fragment, wrap it up in whatever is necessary to turn it into a proper
    /// preview page and write it to a file.
    ///
    /// Input:
    ///
    /// - `content`:    The rendered HTML to write to a file, with some additional metadata.
    /// - `page_state`: Client-side data to embed in the document, so it can read it via javascript
    ///                 and maintain continuity with the previous load.
    ///
    /// Returns the path to the generated HTML file, or an error.
    ///
    pub fn build(&self, content: &RenderedContent, page_state: &PageState) -> anyhow::Result<PathBuf> {
        let output_path     = self.output_path()?;
        let custom_css_path = Config::css_path();

        let json_state = serde_json::to_string(page_state).
            unwrap_or_else(|e| {
                warn!("Couldn't build JSON state from {:?}: {:?}", page_state, e);
                String::from("{}")
            });

        let mut hl_tags = String::new();
        if !content.code_languages.is_empty() {
            let root_url = format!(
                "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/{}",
                HIGHLIGHT_JS_VERSION
            );

            // [Unwrap] Writing to a String should not fail
            writeln!(hl_tags, r#"<link rel="stylesheet" href="{}/styles/github.min.css" />"#, root_url).
                unwrap();
            writeln!(hl_tags, r#"<script src="{}/highlight.min.js"></script>"#, root_url).
                unwrap();

            for language in &content.code_languages {
                writeln!(
                    hl_tags,
                    r#"<script src="{}/languages/{}.min.js"></script>"#,
                    root_url, language
                ).unwrap();
            }

            writeln!(hl_tags, r#"<script>hljs.initHighlighting()</script>"#).unwrap();
        }

        debug!("Building HTML:");
        debug!(" > custom_css_path = {:?}", custom_css_path);
        debug!(" > page_state      = {:?}", json_state);
        debug!(" > code languages  = {:?}", content.code_languages);

        let page = format! {
            include_str!("../res/layout.html"),
            custom_css_path = custom_css_path.display(),
            body            = content.html,
            hl_tags         = hl_tags,
            page_state      = json_state,
        };

        let html_path = output_path.join("index.html");
        fs::write(&html_path, page.as_bytes())?;

        Ok(html_path)
    }

    /// The path on the filesystem where the HTML and other assets go. Could be a temporary
    /// directory, or the one given at construction time.
    ///
    pub fn output_path(&self) -> anyhow::Result<PathBuf> {
        match (&self.real_dir, &self.temp_dir) {
            (Some(path_buf), _) => Ok(path_buf.clone()),
            (_, Some(temp_dir)) => Ok(temp_dir.path().to_path_buf()),
            _ => Err(anyhow!("Assets don't have an output dir, there might be a synchronization error"))
        }
    }

    /// Delete the temporary directory used for building assets, if there is one. This should
    /// happen automatically on drop, but a GTK-level exit doesn't seem to unroll the stack, so we
    /// may need to delete things explicitly.
    ///
    /// If deletion fails, we quietly print a warning. Multiple (successful or failed) deletions
    /// are a noop.
    ///
    pub fn clean_up(&mut self) {
        if let Some(temp_dir) = self.temp_dir.take() {
            let path = temp_dir.path();
            fs::remove_dir_all(path).unwrap_or_else(|_| {
                warn!("Couldn't delete temporary dir: {}", path.display());
            });
        }
    }
}
