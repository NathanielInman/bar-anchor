# bar-anchor

The anchoring crate shared by the waybar apps (roundhouse, tannoy, logbook,
timetable, tablet, waybill, layover). Each pins it by git tag.

- Before a commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- A change that apps should pick up gets a tag: patch for fixes, minor when
  an app's call sites must change. Push the tag with the commit.
- Keep the `Anchor` JSON stable: panels receive it from another process.
- Fixtures under `fixtures/` are real compositor output; add one per new case.

## Platforms

The suite targets Linux, Windows and macOS. This crate builds on all three, and
the pre-push hook checks all three.

- Turn the hook on once per clone: `git config core.hooksPath .githooks`.
  Before a push it runs clippy for every platform listed (needs
  `rustup target add x86_64-pc-windows-gnu aarch64-apple-darwin` and, for
  crates with C code, `mingw-w64-gcc`). A failure there is a broken build on
  that platform: fix it, don't push past it with `--no-verify`.
- Code for one platform goes behind `#[cfg(...)]`, helpers included: a
  function only Linux calls is dead code on Windows, and `-D warnings` fails
  the build there.
- A test that needs something one platform has (a Linux-only binary, Unix
  socket behavior, a POSIX shell, a path layout) is gated with `#[cfg(...)]`
  or skips elsewhere.
