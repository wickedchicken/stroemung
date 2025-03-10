#!/usr/bin/env just --justfile

@default:
    just -f {{ justfile() }} --list --no-aliases

build-release-wasm:
    cargo build --release --target wasm32-unknown-unknown

clippy:
    cargo clippy --all-targets --all-features --workspace -- -D warnings

fmt:
    cargo fmt --all
    just -f {{ justfile() }} --unstable --fmt

serve port="8000": build-release-wasm
    #!/usr/bin/env bash

    # Derived from https://stackoverflow.com/a/34676160/338710

    ### Begin temporary directory creation and destruction block
    readonly TMP_DIR=`mktemp -d 2>/dev/null || mktemp -d -t 'stroemung'`

    # Check if temp directory was created.
    if [[ ! "$TMP_DIR" || ! -d "$TMP_DIR" ]]; then
      echo "Could not create temp dir"
      exit 1
    fi

    # Deletes the temp directory when finished.
    function cleanup {
      echo
      echo "About to remove $TMP_DIR"
      rm -rI "$TMP_DIR"
      echo "Deleted temp working directory $TMP_DIR"
    }

    # Register the cleanup function to be called on the EXIT signal.
    trap cleanup EXIT
    ### End temporary directory creation and destruction block


    ln {{ justfile_directory() }}/assets -s $TMP_DIR/assets
    ln {{ justfile_directory() }}/target/wasm32-unknown-unknown/release/stroemung.wasm -s $TMP_DIR/stroemung.wasm
    ln {{ justfile_directory() }}/html/index.html -s $TMP_DIR/index.html

    echo $TMP_DIR
    echo "Serving at http://[::]:{{ port }}/"
    python3 -m http.server {{ port }} --directory $TMP_DIR

fmt-check:
    cargo fmt --all -- --check
    just -f {{ justfile() }} --unstable --fmt --check

docs $RUSTDOCFLAGS="-D warnings":
    cargo doc --no-deps --document-private-items --all-features --workspace

test:
    cargo test --all-features --workspace

check-and-test: clippy docs fmt-check test
