# Installation and development

## Requirements

- Rust stable toolchain and Cargo.
- Linux desktop session using Wayland or X11.
- Native windowing, graphics and input development/runtime libraries required by eframe on the chosen distribution.
- Native file-dialog support for import/export dialogs.

TH80 Studio uses Rust eframe/egui with the AccessKit accessibility integration enabled. It does not use Electron, Node.js, a browser engine or a WebView. The feature exposes accessibility semantics to supported platform bridges; a formal screen-reader audit is still outstanding.

## Build and run

From the repository root:

    cargo run --locked -p th80-gui
    cargo build --locked -p th80-gui --release

The release binary is target/release/th80-gui.

## Profile storage

Set TH80_PROFILE_DIR to an explicit directory to select an isolated profile repository:

    mkdir -p "$HOME/.local/share/th80-studio-test"
    TH80_PROFILE_DIR="$HOME/.local/share/th80-studio-test" cargo run --locked -p th80-gui

Without that variable, the GUI uses the platform data location. On Linux the default is XDG_DATA_HOME/th80-profiles, or ~/.local/share/th80-profiles when XDG_DATA_HOME is unset. Never point tests at a real profile store.

## Verification

    cargo fmt --all --check
    cargo test --workspace --locked
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo build --workspace --release --locked
    cargo doc --workspace --no-deps --locked

For product-only verification, use -p th80-core -p th80-cli -p th80-gui. The root th80-hid-research and th80-probe packages are separate research tools and are not GUI dependencies.

## Packaging

For a local Linux package, bundle target/release/th80-gui with the applicable license and versioned documentation, and declare the native runtime libraries required by the target distribution. Do not bundle or run the vendor updater. Do not advertise hardware control. No public release or upload is performed by these instructions.

Windows and macOS require native builds and smoke/filesystem verification on their respective platforms; a cross-target compile alone is not equivalent to native validation.
