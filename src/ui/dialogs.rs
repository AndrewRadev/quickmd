//! Modal dialogs.

use std::path::PathBuf;

use gtk::prelude::*;

use crate::input::Config;

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

/// Open a popup that renders documentation for all the default keyboard and mouse mappings.
///
pub fn open_help_dialog(window: &gtk::Window) -> gtk::ResponseType {
    use gtk::{DialogFlags, MessageType, ButtonsType};

    let dialog = gtk::MessageDialog::new(
        Some(window),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        MessageType::Info,
        ButtonsType::Close,
        ""
    );

    let content = format!{
        include_str!("../../res/help_popup.html"),
        yaml_path = Config::yaml_path().display(),
        css_path = Config::css_path().display(),
    };

    dialog.set_markup(&content);
    dialog.connect_response(|d, _response| d.close());

    dialog.run()
}
