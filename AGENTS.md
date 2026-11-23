# bar-anchor

The anchoring crate shared by the waybar apps (roundhouse, tannoy, logbook,
timetable, tablet, waybill, layover). Each pins it by git tag.

- Before a commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- A change that apps should pick up gets a tag: patch for fixes, minor when
  an app's call sites must change. Push the tag with the commit.
- Keep the `Anchor` JSON stable: panels receive it from another process.
- Fixtures under `fixtures/` are real compositor output; add one per new case.
