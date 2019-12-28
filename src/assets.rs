use std::io;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::anyhow;
use dirs::home_dir;
use tempfile::{tempdir, TempDir};
use log::{debug, warn};

const MAIN_JS:    &'static str = include_str!("../res/js/main.js");
const MAIN_CSS:   &'static str = include_str!("../res/style/main.css");
const GITHUB_CSS: &'static str = include_str!("../res/style/github.css");

#[derive(Clone)]
pub struct Assets {
    temp_dir: Option<Rc<TempDir>>,
}

impl Assets {
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

    pub fn build(&self, html: &str, scroll_top: f64) -> anyhow::Result<PathBuf> {
        let temp_dir = self.temp_dir.clone().
            ok_or_else(|| anyhow!("TempDir deleted, there might be a synchronization error"))?;

        let home_path = home_dir().
            map(|p| p.display().to_string()).
            unwrap_or(String::new());

        debug!("Building HTML:");
        debug!(" > home_path  = {}", home_path);
        debug!(" > scroll_top = {}", scroll_top);

        let page = format! {
            include_str!("../res/layout.html"),
            home_path=home_path,
            body=html,
            scroll_top=scroll_top,
        };

        let output_path = temp_dir.path().join("output.html");
        fs::write(&output_path, page.as_bytes())?;

        Ok(output_path)
    }

    pub fn delete(&mut self) {
        if let Some(temp_dir) = self.temp_dir.take() {
            let path = temp_dir.path();
            fs::remove_dir_all(path).unwrap_or_else(|_| {
                warn!("Couldn't delete temporary dir: {}", path.display());
            });
        }
    }
}
