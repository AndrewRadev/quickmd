use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use dirs::home_dir;
use log::{debug, error, warn};
use notify::{Watcher, RecursiveMode, watcher};

use crate::ui;
use crate::markdown;

pub fn init_update_loop(renderer: markdown::Renderer, gui_sender: glib::Sender<ui::Event>) {
    thread::spawn(move || {
        let (watcher_sender, watcher_receiver) = mpsc::channel();

        let mut watcher = match watcher(watcher_sender, Duration::from_millis(200)) {
            Ok(w) => w,
            Err(e) => {
                warn!("Couldn't initialize watcher: {}", e);
                return;
            }
        };
        if let Err(e) = watcher.watch(&renderer.canonical_md_path, RecursiveMode::NonRecursive) {
            warn!("Couldn't initialize watcher: {}", e);
            return;
        }

        if let Some(home) = home_dir() {
            if let Ok(_) = watcher.watch(home.join(".quickmd.css"), RecursiveMode::NonRecursive) {
                debug!("Watching ~/.quickmd.css");
            }
            if let Ok(_) = watcher.watch(home.join(".config/quickmd.css"), RecursiveMode::NonRecursive) {
                debug!("Watching ~/.config/quickmd.css");
            }
        }

        loop {
            use notify::DebouncedEvent::*;

            match watcher_receiver.recv() {
                Ok(Write(file)) => {
                    debug!("File updated: {}", file.display());

                    if file == renderer.canonical_md_path {
                        match renderer.run() {
                            Ok(html) => {
                                let _ = gui_sender.send(ui::Event::LoadHtml(html));
                            },
                            Err(e) => {
                                error! {
                                    "Error rendering markdown ({}): {:?}",
                                    renderer.canonical_md_path.display(), e
                                };
                            }
                        }
                    } else {
                        let _ = gui_sender.send(ui::Event::Reload);
                    }
                },
                Ok(event) => debug!("Ignored watcher event: {:?}", event),
                Err(e) => error!("Error watching file for changes: {:?}", e),
            }
        }
    });
}
