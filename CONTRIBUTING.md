# Contributing to cocoBar

Thank you for your interest in cocoBar!

## Important: No Pull Requests (for now)

**Pull requests are not being accepted at this time.** The project is in early
development and the codebase is changing quickly. Unsolicited pull requests
will likely be closed without review.

## What IS welcome

- **Bug reports** -- open an [Issue](https://github.com/phon-t/CocoBar/issues)
  and include:
  - cocoBar version and Windows version
  - What you did, what you expected, and what actually happened
  - Any error messages or screenshots
- **Feature ideas** -- open an Issue describing what you'd like and why
- **Questions** -- open an Issue with the "question" label

## Coding style (for future contributors)

- Rust, stable edition 2021, `x86_64-pc-windows-gnu` target
- Raw Win32 via `windows-sys` -- no GUI frameworks
- No external files at runtime; assets are embedded via `include_bytes!`
- Follow existing naming and formatting conventions in the codebase
- Run `cargo build --release` and ensure there are no new warnings before submitting

## Build

See the "Build from Source" section in the [README](README.md).

## Code of Conduct

All interactions are governed by our
[Code of Conduct](CODE_OF_CONDUCT.md).
