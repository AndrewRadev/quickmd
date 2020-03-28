//! Markdown rendering.
//!
//! Currently, just uses the `pulldown_cmark` in the simplest of ways.

use std::fs;
use std::io;
use std::path::PathBuf;
use pulldown_cmark::{Parser, html};

/// Encapsulates a markdown file and provides an interface to turn its contents into HTML.
///
pub struct Renderer {
    /// The original path given to the renderer.
    pub md_path: PathBuf,

    /// The canonicalized path to use in file operations.
    pub canonical_md_path: PathBuf,
}

impl Renderer {
    /// Create a new renderer instance that wraps the given markdown file.
    ///
    pub fn new(md_path: PathBuf) -> Self {
        let canonical_md_path = md_path.canonicalize().
            unwrap_or_else(|_| md_path.clone());

        Renderer { md_path, canonical_md_path }
    }

    /// Turn the current contents of the markdown file into HTML.
    ///
    pub fn run(&self) -> Result<String, io::Error> {
        let markdown = fs::read_to_string(&self.canonical_md_path)?;

        let parser = Parser::new(&markdown);
        let mut output = String::new();
        html::push_html(&mut output, parser);
        Ok(output)
    }
}
