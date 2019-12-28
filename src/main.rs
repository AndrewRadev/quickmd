use std::env;
use std::path::PathBuf;
use std::process;

use anyhow::anyhow;

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

fn run() -> anyhow::Result<()> {
    gtk::init()?;

    let input = env::args().nth(1).ok_or_else(|| {
        anyhow!("USAGE: quickmd <file.md>")
    })?;

    let md_path = PathBuf::from(&input);
    if !md_path.exists() {
        let error = anyhow!("File not found: {}", md_path.display());
        return Err(error);
    }
    let renderer = Renderer::new(md_path);

    let ui = ui::App::init()?;

    ui.set_filename(&renderer.display_md_path);
    ui.connect_events();

    let (gui_sender, gui_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    ui::init_render_loop(ui.clone(), gui_receiver);
    background::init_update_loop(renderer, gui_sender);

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
