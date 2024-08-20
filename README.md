# WGPU automatas

Some simple cellular automata examples that run on the gpu

I am using [eframe_template](https://github.com/emilk/eframe_template) since I want to use egui for debug menus. It also handles a lot of boilerplate for me, such as initializing winit and wgpu for both native and web platforms and running the event loop.

## Running

This application can run natively on desktop platforms (Windows, MacOS, and Linux) as well as on the web using wasm.

You can try the wasm version [here](https://gonkalbell.github.io/wgpu_automata/)

Or you can download prebuilt binaries for your OS [here](https://github.com/Gonkalbell/wgpu_automata/releases/tag/main-release)

## Building

Here are the instructions for building and testing locally, either on natively on you machine or in your browser.

### Native

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run` or `cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Wasm

I use [Trunk](https://trunkrs.dev/) to build for web target.

1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` or `trunk serve --release` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.
