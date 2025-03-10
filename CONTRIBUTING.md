# Contribution guidelines

First off, thank you for considering contributing to stroemung.

If your contribution is not straightforward, please first discuss the change you
wish to make by creating a new issue before making the change.

## Reporting issues

Before reporting an issue on the
[issue tracker](https://github.com/wickedchicken/stroemung/issues),
please check that it has not already been reported by searching for some related
keywords.

## Pull requests

Try to do one pull request per change.

### Updating the changelog

Update the changes you have made in
[CHANGELOG](https://github.com/wickedchicken/stroemung/blob/main/CHANGELOG.md)
file under the **Unreleased** section.

Add the changes of your pull request to one of the following subsections,
depending on the types of changes defined by
[Keep a changelog](https://keepachangelog.com/en/1.0.0/):

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

If the required subsection does not exist yet under **Unreleased**, create it!

## Developing

### Set up

This is no different than other Rust projects.

```shell
git clone https://github.com/wickedchicken/stroemung
cd stroemung
cargo test
```

### Useful Commands

Many useful commands are encoded in the `justfile`. The CI system uses the invocations in
the `justfile`, so using them during development gives a higher confidence that things
will pass on CI.

You will need to install [`just`][just] to run them. You can do so via your system package
manager or the `just` releases page.

By default, running `just` by itself will list all actions in the `justfile`:

  ```shell
  just

  ```

- Run all checks and tests:

  ```shell
  just check-and-test
  ```

- Run Clippy:

  ```shell
  just clippy
  ```

- Run all tests:

  ```shell
  just test
  ```

- Check to see if there are code formatting issues

  ```shell
  just fmt-check
  ```

- Format the code in the project

  ```shell
  just fmt
  ```

- Build and check the docs:

  ```shell
  just docs
  ```

- Create a temporary directory with symlinks to `index.html`, `assets`, and
  `stroemung.wasm`, then spawn a webserver with `python3 -m http.server` to allow
  for local development of the web app.

```shell
just serve
```

[just]: https://github.com/casey/just
