//! Input handling.
//!
//! Currently, this only includes handling command-line options. Potentially the place to handle
//! any other type of configuration and input to the application.

use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::io;

use structopt::StructOpt;
use tempfile::NamedTempFile;

/// Command-line options. Managed by StructOpt.
#[derive(Debug, StructOpt)]
#[structopt(name = "quickmd", about = "A simple markdown previewer.")]
pub struct Options {
    /// Activates debug logging
    #[structopt(short, long)]
    pub debug: bool,

    /// Markdown file to render. Use "-" to read markdown from STDIN (implies --no-watch)
    #[structopt(name = "input-file.md", parse(from_os_str))]
    pub input_file: PathBuf,

    /// Disables watching file for changes
    #[structopt(long = "no-watch", parse(from_flag = std::ops::Not::not))]
    pub watch: bool,

    /// Builds output HTML and other assets in the given directory instead of in a tempdir.
    /// Will be created if it doesn't exist. Not deleted on application exit.
    #[structopt(long = "output", name = "directory")]
    pub output_dir: Option<PathBuf>,
}

impl Options {
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
        match self {
            Self::Filesystem(_) => true,
            _                   => false,
        }
    }
}
