//! Input handling.
//!
//! Currently, this only includes handling command-line options. Potentially the place to handle
//! any other type of configuration and input to the application.

use std::fs::File;
use std::io;
use std::path::{PathBuf, Path};
use std::rc::Rc;

use anyhow::anyhow;
use directories::ProjectDirs;
use log::{debug, error};
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use tempfile::NamedTempFile;

use crate::assets::HIGHLIGHT_JS_VERSION;

/// Command-line options. Managed by StructOpt.
#[derive(Debug, StructOpt)]
pub struct Options {
    /// Activates debug logging
    #[structopt(short, long)]
    pub debug: bool,

    /// Markdown file to render. Use "-" to read markdown from STDIN (implies --no-watch)
    #[structopt(name = "input-file.md", parse(from_os_str))]
    pub input_file: Option<PathBuf>,

    /// Disables watching file for changes
    #[structopt(long = "no-watch", parse(from_flag = std::ops::Not::not))]
    pub watch: bool,

    /// Builds output HTML and other assets in the given directory instead of in a tempdir.
    /// Will be created if it doesn't exist. Not deleted on application exit.
    #[structopt(long = "output", name = "directory")]
    pub output_dir: Option<PathBuf>,

    /// Creates a configuration file for later editing if one doesn't exist. Exits when done.
    #[structopt(long)]
    pub install_default_config: bool,
}

impl Options {
    /// Creates a new instance by parsing input args. Apart from just running StructOpt's
    /// initialization, it also adds some additional information to the description that depends on
    /// the current environment.
    ///
    pub fn build() -> Self {
        let description = &[
            "A simple self-contained markdown previewer. ",
            "",
            &format!("Code highlighting via highlight.js version {}", HIGHLIGHT_JS_VERSION),
            "",
            &format!("Edit configuration in: {}", Config::yaml_path().display()),
            &format!("Add custom CSS in:     {}", Config::css_path().display()),
        ].join("\n");

        let options_app = Options::clap().
            long_about(description.as_str());

        Options::from_clap(&options_app.get_matches())
    }

    /// Start logging based on input flags.
    ///
    /// With `--debug`:
    ///   - All logs
    ///   - Timestamps
    ///   - Module path context
    ///
    /// Otherwise:
    ///   - Only warnings and errors
    ///   - Minimal formatting
    ///
    pub fn init_logging(&self) {
        if self.debug {
            env_logger::builder().
                filter_level(log::LevelFilter::Debug).
                init();
        } else {
            env_logger::builder().
                format_module_path(false).
                format_timestamp(None).
                filter_level(log::LevelFilter::Warn).
                init();
        }
    }
}

/// Configuration that controls the behaviour of the app. Saved in a file in the standard app
/// config directory named "config.yaml".
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The zoom level of the page. Defaults to 1.0, but on a HiDPI screen should be set to a
    /// higher value.
    ///
    pub zoom: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

impl Config {
    /// Loads the config from its standard location, or returns None if the file couldn't be found
    /// or is invalid.
    ///
    pub fn load() -> Option<Self> {
        let yaml_path = Self::yaml_path();
        let config_file = File::open(&yaml_path).map_err(|_| {
            debug!("Didn't find config file: {}", yaml_path.display());
        }).ok()?;

        serde_yaml::from_reader(&config_file).map_err(|e| {
            error!("Couldn't parse YAML config file: {}", e);
        }).ok()
    }

    /// Gets the path to the default YAML config in the standard config location.
    pub fn yaml_path() -> PathBuf {
        ProjectDirs::from("com", "andrewradev", "quickmd").
            map(|pd| pd.config_dir().join("config.yaml")).
            unwrap_or_else(|| PathBuf::from("./quickmd.yaml"))
    }

    /// Gets the path to the custom CSS config in the standard config location.
    pub fn css_path() -> PathBuf {
        ProjectDirs::from("com", "andrewradev", "quickmd").
            map(|pd| pd.config_dir().join("custom.css")).
            unwrap_or_else(|| PathBuf::from("./quickmd.css"))
    }

    /// Attempts to install a config file with defaults. Returns an error if a file already exists
    /// in the expected location.
    ///
    pub fn try_install_default() -> anyhow::Result<()> {
        let yaml_path = Config::yaml_path();

        if yaml_path.exists() {
            Err(anyhow!("An existing file was found at: {}\n\
                    If you want to replace it, please delete it first", yaml_path.display()))
        } else {
            Ok(std::fs::write(yaml_path, include_str!("../res/default_config.yaml"))?)
        }
    }
}

/// The file used as input to the application. Could be an existing file path or STDIN.
#[derive(Debug, Clone)]
pub enum InputFile {
    /// A path representing a file on the filesystem.
    Filesystem(PathBuf),

    /// STDIN, written to a named tempfile. It's packaged in an Rc, so we can safely clone the
    /// structure.
    Stdin(Rc<NamedTempFile>),
}

impl InputFile {
    /// Construct an `InputFile` based on the given path.
    ///
    /// If the path is "-", the given `contents` are assumed to be STDIN, they're written down in a
    /// tempfile and returned. Otherwise, that parameter is ignored.
    ///
    pub fn from(path: &Path, mut contents: impl io::Read) -> anyhow::Result<InputFile> {
        if path == PathBuf::from("-") {
            let mut tempfile = NamedTempFile::new()?;
            io::copy(&mut contents, &mut tempfile)?;

            Ok(InputFile::Stdin(Rc::new(tempfile)))
        } else {
            Ok(InputFile::Filesystem(path.to_path_buf()))
        }
    }

    /// Get the path to a real file on the filesystem.
    pub fn path(&self) -> &Path {
        match self {
            Self::Filesystem(path_buf) => path_buf.as_path(),
            Self::Stdin(tempfile)      => tempfile.path(),
        }
    }

    /// Only true if the struct represents an actual file.
    pub fn is_real_file(&self) -> bool {
        matches!(self, Self::Filesystem(_))
    }
}
