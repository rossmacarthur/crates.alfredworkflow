# crates.alfredworkflow

[![Build Status](https://badgers.space/github/checks/rossmacarthur/crates.alfredworkflow?label=build)](https://github.com/rossmacarthur/crates.alfredworkflow/actions/workflows/build.yaml?query=branch%3Atrunk)
[![Latest Release](https://badgers.space/github/release/rossmacarthur/crates.alfredworkflow)](https://github.com/rossmacarthur/crates.alfredworkflow/releases/latest)

📦 Alfred workflow to search Rust crates.

<img style="padding: 1rem 0" width="605" alt="Screenshot" src="https://user-images.githubusercontent.com/17109887/228117520-65aa485d-7252-4766-8fed-724d3a33f93b.gif">

## Features

- Search for crates by name.
- Open the crate in the default browser. You can use modifiers to change the
  URL that is navigated to.
  - **⏎**: opens the crate in https://crates.io.
  - **⌥ ⏎**: opens the crate in https://lib.rs.
  - **⇧ ⏎**: opens the crate in https://docs.rs.
- Manages a local [Crates.io index][crates.io-index].
- Shortcuts for `std`, `core`, and `alloc` crates.
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

## Configuration

The workflow will automatically maintain a local index
[crates.io](crates.io-index) index. The index will be stored in the workflow
cache directory. The update frequency can be configured be setting the
`INDEX_UPDATE_INTERVAL_MINS` environment variable. The default is to update
every 6 hours.

| Name                       | Default | Description                    |
| -------------------------- | ------- | ------------------------------ |
| INDEX_UPDATE_INTERVAL_MINS | 360     | The update interval in minutes |

## Debugging

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

[crates.io-index]: https://github.com/rust-lang/crates.io-index

## License

This project is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
