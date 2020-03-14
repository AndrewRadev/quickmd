use std::path::PathBuf;
use std::process;

use anyhow::anyhow;
use structopt::StructOpt;

use quickmd::markdown::Renderer;
use quickmd::ui;
use quickmd::background;

#[derive(Debug, StructOpt)]
#[structopt(name = "quickmd", about = "A simple markdown previewer.")]
struct Options {
    /// Activate debug logging
    #[structopt(short, long)]
    debug: bool,

    /// Markdown file to render
    #[structopt(name = "input-file.md", parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let options = Options::from_args();

    init_logging(&options);

    if let Err(e) = run(&options) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run(options: &Options) -> anyhow::Result<()> {
    gtk::init()?;

    let md_path = options.input.clone();
    if !md_path.exists() {
        let error = anyhow!("File not found: {}", md_path.display());
        return Err(error);
    }
    let renderer = Renderer::new(md_path);

    let ui = ui::App::init(renderer.display_md_path.to_str())?;
    let (ui_sender, ui_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    ui.init_render_loop(ui_receiver);
    background::init_update_loop(renderer, ui_sender);

    ui.run();
    Ok(())
}

fn init_logging(options: &Options) {
    if options.debug {
        // - All logs
        // - Full info
        env_logger::builder().
            filter_level(log::LevelFilter::Debug).
            init();
    } else {
        // - Only warnings and errors
        // - No timestamps
        // - No module info
        env_logger::builder().
            default_format_module_path(false).
            default_format_timestamp(false).
            filter_level(log::LevelFilter::Warn).
            init();
    }
}
