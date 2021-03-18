//! The GTK user interface.

use std::path::{PathBuf, Path};
use std::process::Command;
use std::time::Instant;

use anyhow::anyhow;
use gdk::keys;
use gio::Cancellable;
use gtk::prelude::*;
use log::{debug, warn};
use pathbuftools::PathBufTools;
use webkit2gtk::{WebContext, WebView, WebViewExt};

use crate::assets::{Assets, PageState};
use crate::input::{InputFile, Config};
use crate::markdown::RenderedContent;

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

        let title = match &input_file {
            InputFile::Filesystem(p) => format!("{} - Quickmd", p.short_path().display()),
            InputFile::Stdin(_)      => String::from("Quickmd"),
        };
        window.set_title(&title);

        let browser = Browser::new(config.clone())?;
        window.add(&browser.webview);

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

        // Key presses mapped to repeatable events:
        let browser = self.browser.clone();
        self.window.connect_key_press_event(move |_window, event| {
            let keyval   = event.get_keyval();
            let keystate = event.get_state();

            match (keystate, keyval) {
                // Scroll with j/k, J/K:
                (_, keys::constants::j) => browser.execute_js("window.scrollBy(0, 70)"),
                (_, keys::constants::J) => browser.execute_js("window.scrollBy(0, 250)"),
                (_, keys::constants::k) => browser.execute_js("window.scrollBy(0, -70)"),
                (_, keys::constants::K) => browser.execute_js("window.scrollBy(0, -250)"),
                // Jump to the top/bottom with g/G
                (_, keys::constants::g) => browser.execute_js("window.scroll({top: 0})"),
                (_, keys::constants::G) => {
                    browser.execute_js("window.scroll({top: document.body.scrollHeight})")
                },
                _ => (),
            }
            Inhibit(false)
        });

        // Key releases mapped to one-time events:
        let browser = self.browser.clone();
        self.window.connect_key_release_event(move |window, event| {
            let keyval   = event.get_keyval();
            let keystate = event.get_state();

            match (keystate, keyval) {
                // Ctrl+Q
                (gdk::ModifierType::CONTROL_MASK, keys::constants::q) => {
                    gtk::main_quit();
                },
                // e:
                (_, keys::constants::e) => {
                    debug!("Launching an editor");
                    launch_editor(&editor_command, &filename);
                },
                // E:
                (_, keys::constants::E) => {
                    debug!("Exec-ing into an editor");
                    exec_editor(&editor_command, &filename);
                },
                // +/-/=:
                (_, keys::constants::plus)  => browser.zoom_in(),
                (_, keys::constants::minus) => browser.zoom_out(),
                (_, keys::constants::equal) => browser.zoom_reset(),
                // F1
                (_, keys::constants::F1) => {
                    build_help_dialog(&window).run();
                },
                _ => (),
            }
            Inhibit(false)
        });

        // On Ctrl+Scroll, zoom:
        let browser = self.browser.clone();
        self.window.connect_scroll_event(move |_window, event| {
            if event.get_state().contains(gdk::ModifierType::CONTROL_MASK) {
                match event.get_direction() {
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

/// A thin layer on top of `webkit2gtk::WebView` to put helper methods into.
///
#[derive(Clone)]
pub struct Browser {
    webview: WebView,
    config: Config,
}

impl Browser {
    /// Construct a new instance with the provided `Config`.
    ///
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let web_context = WebContext::get_default().
            ok_or_else(|| anyhow!("Couldn't initialize GTK WebContext"))?;
        let webview = WebView::with_context(&web_context);
        webview.set_zoom_level(config.zoom);

        Ok(Browser { webview, config })
    }

    /// Delegates to `webkit2gtk::WebView`
    pub fn load_uri(&self, uri: &str) {
        self.webview.load_uri(uri);
    }

    /// Delegates to `webkit2gtk::WebView`
    pub fn reload(&self) {
        self.webview.reload();
    }

    /// Increase zoom level by ~10%
    ///
    pub fn zoom_in(&self) {
        let zoom_level = self.webview.get_zoom_level();
        self.webview.set_zoom_level(zoom_level + 0.1);
        debug!("Zoom level set to: {}", zoom_level);
    }

    /// Decrease zoom level by ~10%, down till 20% or so.
    ///
    pub fn zoom_out(&self) {
        let zoom_level = self.webview.get_zoom_level();

        if zoom_level > 0.2 {
            self.webview.set_zoom_level(zoom_level - 0.1);
            debug!("Zoom level set to: {}", zoom_level);
        }
    }

    /// Reset to the base zoom level defined in the config (which defaults to 100%).
    ///
    pub fn zoom_reset(&self) {
        self.webview.set_zoom_level(self.config.zoom);
        debug!("Zoom level set to: {}", self.config.zoom);
    }

    /// Get the deserialized `PageState` from the current contents of the webview. This is later
    /// rendered unchanged into the HTML content.
    ///
    pub fn get_page_state(&self) -> PageState {
        match self.webview.get_title() {
            Some(t) => {
                serde_json::from_str(t.as_str()).unwrap_or_else(|e| {
                    warn!("Failed to get page state from {}: {:?}", t, e);
                    PageState::default()
                })
            },
            None => PageState::default(),
        }
    }

    /// Execute some (async) javascript code in the webview, without checking the result other than
    /// printing a warning if it errors out.
    ///
    pub fn execute_js(&self, js_code: &'static str) {
        let now = Instant::now();

        self.webview.run_javascript(js_code, None::<&Cancellable>, move |result| {
            if let Err(e) = result {
                warn!("Javascript execution error: {}", e);
            } else {
                debug!("Javascript executed in {}ms:\n> {}", now.elapsed().as_millis(), js_code);
            }
        });
    }
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
        include_str!("../res/help_popup.html"),
        yaml_path = Config::yaml_path().display(),
        css_path = Config::css_path().display(),
    };

    dialog.set_markup(&content);
    dialog.connect_response(|d, _response| d.close());

    dialog
}
