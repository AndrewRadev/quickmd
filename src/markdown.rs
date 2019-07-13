use std::fs;
use std::io;
use std::path::PathBuf;
use pulldown_cmark::{Parser, html};

#[derive(Clone)]
pub struct Content {
    pub md_path: PathBuf,
}

impl Content {
    pub fn new(md_path: PathBuf) -> Self {
        Content { md_path }
    }

    pub fn render(&self) -> Result<String, io::Error> {
        let markdown = fs::read_to_string(&self.md_path)?;

        let parser = Parser::new(&markdown);
        let mut output = String::new();
        html::push_html(&mut output, parser);
        Ok(output)
    }
}
