use std::fs;
use std::io;
use std::path::PathBuf;
use pulldown_cmark::{Parser, html};

pub struct Renderer {
    pub display_md_path: PathBuf,
    pub canonical_md_path: PathBuf,
}

impl Renderer {
    pub fn new(md_path: PathBuf) -> Self {
        let canonical_md_path = md_path.canonicalize().
            unwrap_or_else(|_| md_path.clone());
        let display_md_path = md_path;

        Renderer { display_md_path, canonical_md_path }
    }

    pub fn run(&self) -> Result<String, io::Error> {
        let markdown = fs::read_to_string(&self.canonical_md_path)?;

        let parser = Parser::new(&markdown);
        let mut output = String::new();
        html::push_html(&mut output, parser);
        Ok(output)
    }
}
