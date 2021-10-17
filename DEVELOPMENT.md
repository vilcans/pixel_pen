# Development

Pixel Pen is written in Rust, so install Rust with [Rustup](https://rustup.rs/).

The project uses [Egui](https://github.com/emilk/egui) for the user interface,
and was originally cloned from the [template repo for egui](https://github.com/emilk/egui_template/).

On Linux you need to install some dependencies to build:

    sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libgtk-3-dev

## Testing locally

To build for release and run:

    cargo run --release

The `imagequant` feature is enabled by default and enables the [imagequant](https://crates.io/crates/imagequant) crate, which Pixel Pen uses to convert images to the limited palette of the Vic-20.
If `imagequant` doesn't compile (it's a wrapper for a C++ library), or to avoid the compilation time, you can disable that feature and get a much simpler conversion with the command:

    cargo run --release --no-default-features

## Compiling for the web

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
