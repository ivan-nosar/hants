# Contributing

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
