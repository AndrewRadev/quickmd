use std::io;
use std::process;

use anyhow::anyhow;
use structopt::StructOpt;

use quickmd::markdown::Renderer;
use quickmd::ui;
use quickmd::background;
use quickmd::input::{InputFile, Options};

fn main() {
    let options = Options::from_args();
    options.init_logging();

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

    let mut ui = ui::App::init(input_file.clone())?;
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
