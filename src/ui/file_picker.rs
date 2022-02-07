//! A popup to choose a file if it wasn't provided on the command-line.

use std::path::PathBuf;

use gtk::prelude::*;

/// A popup to choose a file if it wasn't provided on the command-line.
///
pub struct FilePicker(gtk::FileChooserDialog);

impl FilePicker {
    /// Construct a new file picker that only shows markdown files by default
    ///
    pub fn new() -> FilePicker {
        let dialog = gtk::FileChooserDialog::new(
            Some("Open"),
            Some(&gtk::Window::new(gtk::WindowType::Popup)),
            gtk::FileChooserAction::Open,
        );

        // Only show markdown files
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Markdown files (*.md, *.markdown)"));
        filter.add_pattern("*.md");
        filter.add_pattern("*.markdown");
        dialog.add_filter(&filter);

        // Just in case, allow showing all files
        let filter = gtk::FileFilter::new();
        filter.add_pattern("*");
        filter.set_name(Some("All files"));
        dialog.add_filter(&filter);

        // Add the cancel and open buttons to that dialog.
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button("Open", gtk::ResponseType::Ok);

        FilePicker(dialog)
    }

    /// Open the file picker popup and get the selected file.
    ///
    pub fn run(&self) -> Option<PathBuf> {
        if self.0.run() == gtk::ResponseType::Ok {
            self.0.filename()
        } else {
            None
        }
    }
}

impl Drop for FilePicker {
    fn drop(&mut self) { self.0.close(); }
}
