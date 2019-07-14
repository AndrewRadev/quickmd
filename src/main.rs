use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use dirs::home_dir;
use notify::{Watcher, RecursiveMode, watcher};

use quickmd::markdown::Content;
use quickmd::ui;

fn init_watch_loop(content: Content, gui_sender: glib::Sender<ui::Event>) {
    thread::spawn(move || {
        let (watcher_sender, watcher_receiver) = mpsc::channel();
        let mut watcher = watcher(watcher_sender, Duration::from_millis(200)).unwrap();
        watcher.watch(&content.canonical_md_path, RecursiveMode::NonRecursive).unwrap();

        if let Some(home) = home_dir() {
            let _ = watcher.watch(home.join(".quickmd.css"), RecursiveMode::NonRecursive);
            let _ = watcher.watch(home.join(".config/quickmd.css"), RecursiveMode::NonRecursive);
        }

        loop {
            use notify::DebouncedEvent::*;

            match watcher_receiver.recv() {
                Ok(Write(file)) => {
                    if file == content.canonical_md_path {
                        match content.render() {
                            Ok(html) => {
                                let _ = gui_sender.send(ui::Event::LoadHtml(html));
                            },
                            Err(e) => {
                                eprintln! {
                                    "Error rendering markdown ({}): {:?}",
                                    content.canonical_md_path.display(), e
                                };
                            }
                        }
                    } else {
                        let _ = gui_sender.send(ui::Event::Reload);
                    }
                },
                Ok(_) => {
                    // TODO consider "verbose mode" with output
                }
                Err(e) => eprintln!("Error watching file for changes: {:?}", e),
            }
        }
    });
}

fn init_ui_render_loop(mut ui: ui::App, gui_receiver: glib::Receiver<ui::Event>) {
    gui_receiver.attach(None, move |event| {
        match event {
            ui::Event::LoadHtml(html) => ui.load_html(&html),
            ui::Event::Reload => ui.reload(),
        }
        glib::Continue(true)
    });
}

fn main() {
    better_panic::install();

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

    let md_path = PathBuf::from(&input);
    let content = Content::new(md_path);

    let mut ui = ui::App::init();
    let html = content.render().map_err(|e| {
        format!("Couldn't parse markdown from file {}: {}", content.canonical_md_path.display(), e)
    })?;

    ui.set_filename(&content.display_md_path);
    ui.connect_events();
    ui.load_html(&html);

    let (gui_sender, gui_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    init_watch_loop(content.clone(), gui_sender);
    init_ui_render_loop(ui.clone(), gui_receiver);

    ui.run();
    Ok(())
}
