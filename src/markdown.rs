//! Markdown rendering.
//!
//! Currently, just uses the `pulldown_cmark` in the simplest of ways.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::collections::HashSet;
use pulldown_cmark::{Parser, Options, Event, html};

/// The output of the rendering process. Includes both the rendered HTML and additional metadata
/// used by its clients.
///
#[derive(Debug, Default)]
pub struct RenderedContent {
    /// The rendered HTML.
    pub html: String,

    /// All the languages in fenced code blocks from the markdown input.
    pub code_languages: HashSet<String>,
}

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
    pub fn run(&self) -> Result<RenderedContent, io::Error> {
        let markdown = fs::read_to_string(&self.canonical_md_path)?;

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&markdown, options);

        let mut languages = HashSet::new();
        let parser = parser.map(|event| {
            use pulldown_cmark::{Tag, CodeBlockKind};

            if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(content))) = &event {
                if content.len() > 0 {
                    languages.insert(content.to_string());
                }
            }

            event
        });

        let mut output = String::new();
        html::push_html(&mut output, parser);

        Ok(RenderedContent {
            html: output,
            code_languages: languages,
        })
    }
}
