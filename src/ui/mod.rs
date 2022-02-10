//! The GTK user interface.

pub mod action;
pub mod browser;
pub mod file_picker;

use std::path::{PathBuf, Path};
use std::process::Command;

use gtk::prelude::*;
use log::{debug, warn, error};
use pathbuftools::PathBufTools;

use crate::assets::Assets;
use crate::input::{InputFile, Config};
use crate::markdown::RenderedContent;
use crate::ui::browser::Browser;
use crate::ui::action::{Action, Keymaps};

/// The container for all the GTK widgets of the app -- window, webview, etc.
/// All of these are reference-counted, so should be cheap to clone.
///
#[derive(Clone)]
pub struct App {
    window: gtk::Window,
    browser: Browser,
    assets: Assets,
    filename: PathBuf,
    config: Config,
}

impl App {
    /// Construct a new app. Input params:
    ///
    /// - input_file: Used as the window title and for other actions on the file.
    /// - assets:     Encapsulates the HTML layout that will be wrapping the rendered markdown.
    ///
    /// Initialization could fail due to a `WebContext` failure.
    ///
    pub fn init(config: Config, input_file: InputFile, assets: Assets) -> anyhow::Result<Self> {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_default_size(1024, 768);

        if let Ok(asset_path) = assets.output_path() {
            if let Ok(icon) = gdk_pixbuf::Pixbuf::from_file(asset_path.join("icon.png")) {
                window.set_icon(Some(&icon));
            }
        }

        let title = match &input_file {
            InputFile::Filesystem(p) => format!("{} - Quickmd", p.short_path().display()),
            InputFile::Stdin(_)      => String::from("Quickmd"),
        };
        window.set_title(&title);

        let browser = Browser::new(config.clone())?;
        browser.attach_to(&window);

        Ok(App { window, browser, assets, config, filename: input_file.path().to_path_buf() })
    }

    /// Start listening to events from the `ui_receiver` and trigger the relevant methods on the
    /// `App`. Doesn't block.
    ///
    pub fn init_render_loop(&self, ui_receiver: glib::Receiver<Event>) {
        let mut app_clone = self.clone();

        ui_receiver.attach(None, move |event| {
            match event {
                Event::LoadHtml(content) => {
                    app_clone.load_content(&content).
                        unwrap_or_else(|e| warn!("Couldn't update HTML: {}", e))
                },
                Event::Reload => app_clone.reload(),
            }
            glib::Continue(true)
        });
    }

    /// Actually start the UI, blocking the main thread.
    ///
    pub fn run(&mut self) {
        self.connect_events();
        self.window.show_all();

        gtk::main();

        self.assets.clean_up();
    }

    fn load_content(&mut self, content: &RenderedContent) -> anyhow::Result<()> {
        let page_state = self.browser.get_page_state();
        let output_path = self.assets.build(content, &page_state)?;

        debug!("Loading HTML:");
        debug!(" > output_path = {}", output_path.display());

        self.browser.load_uri(&format!("file://{}", output_path.display()));
        Ok(())
    }

    fn reload(&self) {
        self.browser.reload();
    }

    fn connect_events(&self) {
        let filename        = self.filename.clone();
        let editor_command  = self.config.editor_command.clone();

        let mut keymaps = Keymaps::default();
        keymaps.add_config_mappings(&self.config.mappings).unwrap_or_else(|e| {
            error!("Mapping parsing error: {}", e);
        });

        // Key presses mapped to repeatable events:
        let browser = self.browser.clone();
        let keymaps_clone = keymaps.clone();
        self.window.connect_key_press_event(move |_window, event| {
            let keyval   = event.keyval();
            let keystate = event.state();

            match keymaps_clone.get_action(keystate, keyval) {
                Action::SmallScrollDown => browser.execute_js("window.scrollBy(0, 70)"),
                Action::BigScrollDown   => browser.execute_js("window.scrollBy(0, 250)"),
                Action::SmallScrollUp   => browser.execute_js("window.scrollBy(0, -70)"),
                Action::BigScrollUp     => browser.execute_js("window.scrollBy(0, -250)"),
                Action::ScrollToTop     => browser.execute_js("window.scroll({top: 0})"),
                Action::ScrollToBottom  => {
                    browser.execute_js("window.scroll({top: document.body.scrollHeight})")
                },
                _ => (),
            }
            Inhibit(false)
        });

        // Key releases mapped to one-time events:
        let browser = self.browser.clone();
        let keymaps_clone = keymaps.clone();
        self.window.connect_key_release_event(move |window, event| {
            let keyval   = event.keyval();
            let keystate = event.state();

            match keymaps_clone.get_action(keystate, keyval) {
                Action::LaunchEditor => {
                    debug!("Launching an editor");
                    launch_editor(&editor_command, &filename);
                },
                Action::ExecEditor => {
                    debug!("Exec-ing into an editor");
                    exec_editor(&editor_command, &filename);
                },
                Action::ZoomIn    => browser.zoom_in(),
                Action::ZoomOut   => browser.zoom_out(),
                Action::ZoomReset => browser.zoom_reset(),
                Action::ShowHelp  => { build_help_dialog(window).run(); },
                Action::Quit      => gtk::main_quit(),
                _ => (),
            }
            Inhibit(false)
        });

        // On Ctrl+Scroll, zoom:
        let browser = self.browser.clone();
        self.window.connect_scroll_event(move |_window, event| {
            if event.state().contains(gdk::ModifierType::CONTROL_MASK) {
                match event.direction() {
                    gdk::ScrollDirection::Up   => browser.zoom_in(),
                    gdk::ScrollDirection::Down => browser.zoom_out(),
                    _ => (),
                }
            }

            Inhibit(false)
        });

        self.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
    }
}

/// Events that trigger UI changes.
///
#[derive(Debug)]
pub enum Event {
    /// Load the given content into the webview.
    LoadHtml(RenderedContent),

    /// Refresh the webview.
    Reload,
}

#[cfg(target_family="unix")]
fn exec_editor(editor_command: &[String], file_path: &Path) {
    if let Some(mut editor) = build_editor_command(editor_command, file_path) {
        gtk::main_quit();

        use std::os::unix::process::CommandExt;

        editor.exec();
    }
}

#[cfg(not(target_family="unix"))]
fn exec_editor(_editor_command: &[String], _filename_string: &Path) {
    warn!("Not on a UNIX system, can't exec to a text editor");
}

fn launch_editor(editor_command: &[String], file_path: &Path) {
    if let Some(mut editor) = build_editor_command(editor_command, file_path) {
        if let Err(e) = editor.spawn() {
            warn!("Couldn't launch editor ({:?}): {}", editor_command, e);
        }
    }
}

fn build_editor_command(editor_command: &[String], file_path: &Path) -> Option<Command> {
    let executable = editor_command.get(0).or_else(|| {
        warn!("No \"editor\" defined in the config ({})", Config::yaml_path().display());
        None
    })?;

    let mut command = Command::new(executable);

    for arg in editor_command.iter().skip(1) {
        if arg == "{path}" {
            command.arg(file_path);
        } else {
            command.arg(arg);
        }
    }

    Some(command)
}

fn build_help_dialog(window: &gtk::Window) -> gtk::MessageDialog {
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

    dialog
}
