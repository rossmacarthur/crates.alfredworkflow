# crates.alfredworkflow

[![Build status](https://github.com/rossmacarthur/crates.alfredworkflow/actions/workflows/build.yaml/badge.svg)](https://github.com/rossmacarthur/crates.alfredworkflow/actions/workflows/build.yaml)

Alfred workflow to search Rust crates.

<img width="605" alt="Screenshot" src="https://user-images.githubusercontent.com/17109887/116975522-10c55c00-acc0-11eb-856d-e6145d49eebc.png">

## Features

- Manages a local [Crates.io registry](https://github.com/rust-lang/crates.io-index).
- Opens the crate in the default browser. You can use modifiers to change the
  URL that is navigated to.
  - **⏎**: open the crate in https://crates.io.
  - **⌥ ⏎**: open the crate in https://lib.rs.
  - **⇧ ⏎**: open the crate in https://docs.rs.
- Blazingly fast 🤸 (it's built in Rust 🦀).

## 📦 Installation

### Pre-packaged

Grab the latest release from
[the releases page](https://github.com/rossmacarthur/crates.alfredworkflow/releases).

Because the release contains an executable binary later versions of macOS will
mark it as untrusted and Alfred won't be able to execute it. You can run the
following to explicitly trust the release before installing to Alfred.
```sh
xattr -c ~/Downloads/crates-*-x86_64-apple-darwin.alfredworkflow
```

### Building from source

This workflow is written in Rust, so to install it from source you will first
need to install Rust and Cargo using [rustup](https://rustup.rs/). Then install
[powerpack](https://github.com/rossmacarthur/powerpack). Then you can run the
following to build an `.alfredworkflow` file.

```sh
git clone https://github.com/rossmacarthur/crates.alfredworkflow.git
cd crates.alfredworkflow
powerpack package
```

The release will be available at `target/workflow/crates.alfredworkflow`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
