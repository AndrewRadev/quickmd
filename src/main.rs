use gtk::prelude::*;
use gtk::{Window, WindowType};
use webkit2gtk::{WebContext, WebView, WebViewExt};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Quickmd: TODO document title");
    //window.set_default_size(350, 70);

    // Create a the WebView for the preview pane.
    let context = WebContext::get_default().unwrap();
    let preview = WebView::new_with_context(&context);
    preview.load_html("<h1>Test <u>Underlined</u></h1>", Some("file://example.html"));

    window.add(&preview);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
