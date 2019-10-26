use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process;

use quickmd::markdown::Renderer;
use quickmd::ui;
use quickmd::background;

fn main() {
    init_logging();

    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    gtk::init()?;

    let input = env::args().nth(1).ok_or_else(|| {
        format!("USAGE: quickmd <file.md>")
    })?;

    let md_path  = PathBuf::from(&input);
    let renderer = Renderer::new(md_path);

    let mut ui = ui::App::init()?;
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

    ui.run();
    Ok(())
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
