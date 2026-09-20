# Contributing

## Building

Build an optimized binary with the release profile:

```sh
cargo build --release --verbose
```

## Linting

Run Clippy for all targets and features:

```sh
cargo clippy --all-targets --all-features
```

Apply Clippy recommendations for all targets and features:

```sh
cargo clippy --all-targets --all-features --allow-dirty --fix
```

Check formatting without changing files:

```sh
cargo fmt --all -- --check
```

Format the workspace in place:

```sh
cargo fmt --all
```

## Testing

Run the complete test suite:

```sh
cargo test --all-targets --all-features
```

## Testing with Coverage

Install [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) once:

```sh
cargo install cargo-llvm-cov
```

Run the test suite with coverage instrumentation, then print a summary:

```sh
cargo llvm-cov --no-report
cargo llvm-cov report --summary-only
```

To generate a browsable HTML report instead, run:

```sh
cargo llvm-cov --html --open
```

## Releasing a New Version

HANTS uses `cargo dist` crate to support binary release process. Follow the [cargo-dist Rust quickstart](https://axodotdev.github.io/cargo-dist/book/quickstart/rust.html) and create a release with a tag in `**[0-9]+.[0-9]+.[0-9]+*` format:

```sh
# <manually update the version of your crate, run tests, etc>

# commit and push to main (can be done with a PR)
git commit -m "release: version 0.1.0"
git push

# actually push the tag up (this triggers dist's CI)
git tag v0.1.0
git push --tags
```

The `dist`'s self-generated CI is triggered by pushing git tags with specific formats like `v1.0.0`, `my-app-v1.0.0` or `my-app/v1.0.0`. Each tag will trigger its own independent run of that CI workflow.
