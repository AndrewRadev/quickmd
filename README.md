[![Crate](https://img.shields.io/crates/v/quickmd)](https://crates.io/crates/quickmd)
[![Documentation](https://docs.rs/quickmd/badge.svg)](https://docs.rs/quickmd)
[![Maintenance status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)](https://crates.io/crates/quickmd)

# QuickMD

This project is a simple tool that solves a simple problem: I'd like to preview markdown files as they'll show up on Github, so I don't have to push my READMEs before I can tell whether they're formatted well enough. It ends up looking like this:

![Screenshot](http://i.andrewradev.com/d32c593abdfd03881de3358251596bdd.png)

It's a Rust app that launches a GtkWebkit window that renders the compiled HTML of the given markdown file. It monitors this file for any changes and reloads. It uses a stylesheet that's literally copied off Github's markdown stylesheet.

_Note: I have no idea if I'm allowed to use Github's stylesheet. The relevant file is in res/style/github.css, and if I am told I shouldn't be using it I'll go ahead and scrub it from git history._

## Installation

You'll need to have Rust and the `cargo` tool. The easiest way to get that done is through [rustup.rs](https://rustup.rs/).

You'll also need the GTK+, GLib and webkit2gtk development files to be installed on your system. The Gtk-rs [requirements page](http://gtk-rs.org/docs/requirements.html) should be a good guide.

After that, you can build and install the app from `crates.io` using:

```
cargo install quickmd
```

Make sure that `~/.cargo/bin` is in your `PATH` so you can call the `quickmd` executable.

## Usage

Running the app is as simple as:

```
quickmd <markdown-file>
```

Pressing escape will close the window. Running it with `--help` should provide more info on the available options. Here's how the output looks for me:

```
quickmd 0.3.1
A simple self-contained markdown previewer.

Code highlighting via highlight.js version 9.18.1

Edit configuration in: /home/andrew/.config/quickmd/config.yaml
Add custom CSS in:     /home/andrew/.config/quickmd/custom.css

USAGE:
    quickmd [FLAGS] [OPTIONS] [input-file.md]

FLAGS:
    -d, --debug
            Activates debug logging

    -h, --help
            Prints help information

        --install-default-config
            Creates a configuration file for later editing if one doesn't exist. Exits when done

    -V, --version
            Prints version information

        --no-watch
            Disables watching file for changes


OPTIONS:
        --output <directory>
            Builds output HTML and other assets in the given directory instead of in a tempdir. Will be created if it
            doesn't exist. Not deleted on application exit

ARGS:
    <input-file.md>
            Markdown file to render. Use "-" to read markdown from STDIN (implies --no-watch)
```

## Features

- Github-like rendering, though not guaranteed to be perfectly identical. Relying on whatever [pulldown-cmark](https://crates.io/crates/pulldown-cmark) provides, which is good enough for me.

- Fast and seamless preview updates on file write.

- Code highlighting via [highlight.js](https://highlightjs.org/). Currently, the relevant javascript is included via a CDN, which unfortunately means it won't work without an internet connection.

- Ability to render STDIN, which allows partial rendering of target markdown. Try putting [this bit of Vimscript](https://github.com/AndrewRadev/Vimfiles/blob/f9e0c08dd280d13acb625d3370da399c39e14403/ftplugin/markdown.vim#L11-L15) in your `~/.vim/ftplugin/markdown.vim`, select a few lines and press `!`.

## Configuration

You can change the CSS of the preview HTML by writing a file named "custom.css" in the application's config directory. On a linux machine, it would be: `~/.config/quickmd/`.

You can also change some configuration options in a config file. Run `quickmd` with `--install-default-config` to create that file with all the defaults and comments.

Run `--help` to know where the config files will be located on your system.

The built-in CSS that is used is stored in [/res/style](./res/style) and the default config is in [/res/default_config.yaml](./res/default_config.yaml)
