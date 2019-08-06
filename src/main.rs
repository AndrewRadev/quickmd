use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process;

use gio::prelude::{ApplicationExt, ApplicationExtManual};

use quickmd::markdown::Renderer;
use quickmd::ui;
use quickmd::background;

const APP_NAME: &'static str = "com.andrewradev.quickmd";

fn main() {
    init_logging();

    let application = gtk::Application::new(Some(APP_NAME), Default::default()).
        expect("GTK initialization failed");

    application.connect_activate(|app| {
        if let Err(e) = run(&app) {
            eprintln!("{}", e);
            process::exit(1);
        }
    });

    process::exit(application.run(&[]));
}

fn init_logging() {
    // Release logging:
    // - Warnings and errors
    // - No timestamps
    // - No module info
    //
    #[cfg(not(debug_assertions))]
    env_logger::builder().
        default_format_module_path(false).
        default_format_timestamp(false).
        filter_level(log::LevelFilter::Warn).
        init();

    // Debug logging:
    // - All logs
    // - Full info
    //
    #[cfg(debug_assertions)]
    env_logger::builder().
        filter_level(log::LevelFilter::Debug).
        init();
}

fn run(gtk_app: &gtk::Application) -> Result<(), Box<dyn Error>> {
    let input = env::args().nth(1).ok_or_else(|| {
        format!("USAGE: quickmd <file.md>")
    })?;

    let md_path  = PathBuf::from(&input);
    let renderer = Renderer::new(md_path);

    let mut ui = ui::MainWindow::open(gtk_app)?;
    let html = renderer.run().map_err(|e| {
        format!("Couldn't parse markdown from file {}: {}", renderer.canonical_md_path.display(), e)
    })?;

    ui.set_filename(&renderer.display_md_path);
    ui.connect_events();
    ui.load_html(&html).map_err(|e| {
        format!("Couldn't load HTML in the UI: {}", e)
    })?;

    let (gui_sender, gui_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    ui::init_render_loop(ui.clone(), gui_receiver);
    background::init_update_loop(renderer.clone(), gui_sender);

    ui.show();
    Ok(())
}
