use std::io;
use std::process;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use log::debug;

use quickmd::assets::Assets;
use quickmd::background;
use quickmd::input::{Config, Options, InputFile};
use quickmd::markdown::Renderer;
use quickmd::ui;

fn main() {
    let config = Config::load().
        unwrap_or_default();

    let options = Options::build();
    options.init_logging();

    debug!("Loaded config: {:?}", config);
    debug!("  > path: {}", Config::yaml_path().display());
    debug!("Using input options: {:?}", options);

    if let Err(e) = run(&config, &options) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run(config: &Config, options: &Options) -> anyhow::Result<()> {
    if options.install_default_config {
        return Config::try_install_default();
    }

    gtk::init()?;

    if let Some(input_file) = options.input_file.as_ref() {
        launch_app(input_file, options, config)
    } else {
        let input_file = launch_file_picker()?;
        launch_app(&input_file, options, config)
    }
}

fn launch_file_picker() -> anyhow::Result<PathBuf> {
    ui::file_picker::FilePicker::new().run().ok_or_else(|| {
        anyhow!("Please provide a markdown file to render or call the program with - to read from STDIN")
    })
}

fn launch_app(input_file: &Path, options: &Options, config: &Config) -> anyhow::Result<()> {
    let input_file   = InputFile::from(input_file, io::stdin())?;
    let is_real_file = input_file.is_real_file();
    let md_path      = input_file.path();

    if !md_path.exists() {
        let error = anyhow!("File not found: {}", md_path.display());
        return Err(error);
    }

    let renderer = Renderer::new(md_path.to_path_buf());
    let assets = Assets::init(options.output_dir.clone())?;

    let mut ui = ui::App::init(config.clone(), input_file.clone(), assets)?;
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
