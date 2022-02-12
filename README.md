# crates.alfredworkflow

[![Build status](https://img.shields.io/github/workflow/status/rossmacarthur/crates.alfredworkflow/build/trunk)](https://github.com/rossmacarthur/crates.alfredworkflow/actions?query=workflow%3Abuild)
[![Latest release](https://img.shields.io/github/v/release/rossmacarthur/crates.alfredworkflow)](https://github.com/rossmacarthur/crates.alfredworkflow/releases/latest)

📦 Alfred workflow to search Rust crates.

<img width="605" alt="Screenshot" src="https://user-images.githubusercontent.com/17109887/116975522-10c55c00-acc0-11eb-856d-e6145d49eebc.png">

## Features

- Search for crates by name.
- Open the crate in the default browser. You can use modifiers to change the
  URL that is navigated to.
  - **⏎**: opens the crate in https://crates.io.
  - **⌥ ⏎**: opens the crate in https://lib.rs.
  - **⇧ ⏎**: opens the crate in https://docs.rs.
- Manages a local [Crates.io index](https://github.com/rust-lang/crates.io-index).
- Blazingly fast 🤸.

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

### Debugging issues

If you are experiencing issues you can debug the workflow in the following way:

1. Inspect the output of the workflow by enabling debug mode in Alfred for the
   workflow.

2. The index is maintained asynchronously and will output any updates and errors
   to a log file in the Alfred cache directly under the bundle name
   `io.macarthur.ross.crates`. The default Alfred cache directory is
  `~/Library/Caches/com.runningwithcrayons.Alfred/Workflow\ Data/io.macarthur.ross.crates`.
  Expected logs will look like the following.
  ```
  [2022-01-31T11:10:24] [INFO] updated index ./crates.io-index: HEAD is now at 603fff76b2 Updating crate `midpoint#0.1.2`
  [2022-02-04T15:06:07] [INFO] updated index ./crates.io-index: HEAD is now at 93d0942359 Updating crate `os_info_cli#2.0.0`
  [2022-02-06T14:41:29] [INFO] updated index ./crates.io-index: HEAD is now at 5864e33978 Updating crate `agsol-gold-bot#0.0.0-alpha.2`
  ```

3. Open an [issue](https://github.com/rossmacarthur/crates.alfredworkflow/issues/new)
   on this repo.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
