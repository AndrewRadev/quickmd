use std::io;
use std::process;

use anyhow::anyhow;
use log::debug;
use structopt::StructOpt;

use quickmd::assets::Assets;
use quickmd::background;
use quickmd::input::{InputFile, Options, Config};
use quickmd::markdown::Renderer;
use quickmd::ui;

fn main() {
    let options = Options::from_args();
    debug!("Using input options: {:?}", options);

    options.init_logging();

    let config = Config::load().
        unwrap_or_else(Config::default);
    debug!("Loaded config: {:?}", config);

    if let Err(e) = run(&options) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run(options: &Options) -> anyhow::Result<()> {
    gtk::init()?;

    let input_file   = InputFile::from(&options.input_file, io::stdin())?;
    let is_real_file = input_file.is_real_file();
    let md_path      = input_file.path();

    if !md_path.exists() {
        let error = anyhow!("File not found: {}", md_path.display());
        return Err(error);
    }
    let renderer = Renderer::new(md_path.to_path_buf());
    let assets = Assets::init(options.output_dir.clone())?;

    let mut ui = ui::App::init(input_file.clone(), assets)?;
    let (ui_sender, ui_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    ui.init_render_loop(ui_receiver);

    // Initial render
    ui_sender.send(ui::Event::LoadHtml(renderer.run()?))?;

    if is_real_file && options.watch {
        background::init_update_loop(renderer, ui_sender);
    }

    ui.run();
    Ok(())
}
