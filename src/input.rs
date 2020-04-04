//! Input handling
//!
//! Currently, this only includes handling command-line options. Potentially the place to handle
//! any other type of configuration and input to the application.

use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::io;

use structopt::StructOpt;
use tempfile::NamedTempFile;

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
    /// Get the path to a real file on the filesystem.
    pub fn path(&self) -> &Path {
        match self {
            Self::Filesystem(path_buf) => path_buf.as_path(),
            Self::Stdin(tempfile)      => tempfile.path(),
        }
    }

    /// Only true if the struct represents an actual file.
    pub fn is_real_file(&self) -> bool {
        match self {
            Self::Filesystem(_) => true,
            _                   => false,
        }
    }
}

/// Command-line options. Managed by StructOpt.
#[derive(Debug, StructOpt)]
#[structopt(name = "quickmd", about = "A simple markdown previewer.")]
pub struct Options {
    /// Activates debug logging
    #[structopt(short, long)]
    pub debug: bool,

    /// Markdown file to render. Use "-" to read markdown from STDIN (implies --no-watch)
    #[structopt(name = "input-file.md", parse(from_os_str))]
    pub input: PathBuf,

    /// Disables watching file for changes
    #[structopt(long = "no-watch", parse(from_flag = std::ops::Not::not))]
    pub watch: bool,
}

impl Options {
    /// Start logging based on input flags.
    ///
    /// With --debug:
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

    /// Get an `InputFile` based on the given command-line options.
    ///
    /// If the file's name is "-", the given `contents` are assumed to be STDIN, they're written
    /// down in a tempfile and returned. Otherwise, that parameter is ignored.
    ///
    pub fn get_input_file(&self, mut contents: impl io::Read) -> anyhow::Result<InputFile> {
        if self.input == PathBuf::from("-") {
            let mut tempfile = NamedTempFile::new()?;
            io::copy(&mut contents, &mut tempfile)?;

            Ok(InputFile::Stdin(Rc::new(tempfile)))
        } else {
            Ok(InputFile::Filesystem(self.input.clone()))
        }
    }
}