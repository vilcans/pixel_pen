# Pixel Pen

*Graphics editor for pictures compatible with Vic-20 hardware.*

Current version: 0.2.2 (2021-10-07)

This is still early in development.

You can load images in typical image file formats, which Pixel Pen will convert to Vic-20 format.
If you use the "Open" menu to open an image file, make sure it's not very high resolution. You can use the Import menu instead to load an image and scale it down at the same time.

It's also possible to load some of the files using the FLUFF64 file format (file extension: `flf`) used by [Turbo Rascal](https://lemonspawn.com/turbo-rascal-syntax-error-expected-but-begin/) (TRSE),
but that was only to get some images to test with at the start of the project. It's not a high priority to support all the formats of TRSE.

There is also Pixel Pen's own file format (file extension: `pixelpen`), the details of which are subject to change, but it's based on JSON. Pixel Pen can save and load files in this format.

Apart from that, you can paint single pixels, and that's about it right now.

## Input

| Input                               | Action
| ----------------------------------- | ---------------------------------
| Left mouse button                   | Paint
| Right mouse button                  | Paint with background color
| +                                   | Zoom in
| -                                   | Zoom out
| Scroll wheel                        | Scroll
| Ctrl + scroll wheel                 | Zoom
| Hold middle mouse button            | Pan
| Hold shift + right mouse button     | Pan (for systems without a middle mouse button or where it scrolls instead)

On a Mac, substitute Ctrl for ⌘.

## Command-line interface

Oh, and there's a command-line interface! Run `pixel_pen --help` to get the possible commands:

    Pixel Pen 0.1.0
    Actual 8 bit graphics editor

    USAGE:
        pixel_pen [OPTIONS] [filename]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
            --export <export-file>    Save the image to the given file and quit
            --import <import-file>    Open the given file in import mode

    ARGS:
        <filename>    File to load

So it can be used to convert images from any supported format to Pixel Pen's format (the only supported export format currently).

## Development

Pixel Pen is written in Rust, so install Rust with [Rustup](https://rustup.rs/).

The project uses [Egui](https://github.com/emilk/egui) for the user interface,
and was originally cloned from the [template repo for egui](https://github.com/emilk/egui_template/).

On Linux you need to install some dependencies to build:

    sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libgtk-3-dev

### Testing locally

To build for release and run:

    cargo run --release

The `imagequant` feature is enabled by default and enables the [imagequant](https://crates.io/crates/imagequant) crate, which Pixel Pen uses to convert images to the limited palette of the Vic-20.
If `imagequant` doesn't compile (it's a wrapper for a C++ library), or to avoid the compilation time, you can disable that feature and get a much simpler conversion with the command:

    cargo run --release --no-default-features

### Compiling for the web

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

Note that this doesn't support loading or saving files, so it's not very useful right now. Consider it a demo version of the full application!

To build for the web you need to set up some tools. There are a few simple scripts that help you with this:

``` sh
./setup_web.sh
./build_web.sh
./start_server.sh
open http://127.0.0.1:8080/
```

* `setup_web.sh` installs the tools required to build for web
* `build_web.sh` compiles your code to wasm and puts it in the `docs/` folder (see below)
* `start_server.sh` starts a local HTTP server so you can test before you publish
* Open http://127.0.0.1:8080/ in a web browser to view

The finished web app is found in the `docs/` folder (this is so that you can easily share it with [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)). It consists of three files:

* `index.html`: A few lines of HTML, CSS and JS that loads your app.
* `pixel_pen_bg.wasm`: What the Rust code compiles to.
* `pixel_pen.js`: Auto-generated binding between Rust and JS.
