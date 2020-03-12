use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::marker::Send;

use dirs::home_dir;
use log::{debug, error, warn};
use notify::{Watcher, RecursiveMode, DebouncedEvent, watcher};

use crate::ui;
use crate::markdown;

pub trait Sender<T> {
    fn send(&mut self, event: T) -> Result<(), mpsc::SendError<T>>;
}

impl<T> Sender<T> for glib::Sender<T> {
    fn send(&mut self, event: T) -> Result<(), mpsc::SendError<T>> {
        glib::Sender::<T>::send(self, event)
    }
}

impl<T> Sender<T> for mpsc::Sender<T> {
    fn send(&mut self, event: T) -> Result<(), mpsc::SendError<T>> {
        mpsc::Sender::<T>::send(self, event)
    }
}

pub fn init_update_loop<S>(renderer: markdown::Renderer, mut gui_sender: S)
    where S: Sender<ui::Event> + Send + 'static
{
    thread::spawn(move || {
        let (watcher_sender, watcher_receiver) = mpsc::channel();

        // Initial render
        if let Err(e) = watcher_sender.send(DebouncedEvent::Write(renderer.canonical_md_path.clone())) {
            error!("Couldn't render markdown: {}", e);
        }

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
            match watcher_receiver.recv() {
                Ok(DebouncedEvent::Write(file)) => {
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
