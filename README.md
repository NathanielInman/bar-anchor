# bar-anchor

Open a panel from the waybar icon that was clicked. Waybar doesn't say where
its modules are, so this asks the compositor instead: where the cursor is,
which monitor it's on and which edge the bar sits on. Hyprland knows the
cursor, so the panel opens at the icon; sway and niri know only the monitor,
so the panel opens on that monitor's bar where a `BarHint` from the bar's
config says the icon is. `place` turns the answer into a panel rectangle and
a scale origin, so the open animation grows out of the icon.

It's the shared crate behind the drawers of
[Roundhouse](https://github.com/NathanielInman/roundhouse),
[Tannoy](https://github.com/NathanielInman/tannoy),
[Logbook](https://github.com/NathanielInman/logbook),
[Timetable](https://github.com/NathanielInman/timetable),
[Tablet](https://github.com/NathanielInman/tablet),
[Waybill](https://github.com/NathanielInman/waybill) and
[Layover](https://github.com/NathanielInman/layover). Each one used to keep
its own copy; by November 2026 there were four versions of it.

```toml
[dependencies]
bar-anchor = { git = "https://github.com/NathanielInman/bar-anchor", tag = "v0.1.0" }
```

```rust
// The drawer's layer-shell namespace, a hint for sway and niri, and whether
// to center the panel instead of opening it at the icon.
let anchor = bar_anchor::anchor("tannoy", bar_anchor::BarHint::default(), false);
// The screen's size, the panel's size and the gap to keep from the bar.
let (rect, origin) = bar_anchor::place(anchor.as_ref(), screen_w, screen_h, 420.0, 360.0, 8.0);
```

The `Anchor` JSON is the wire format Switchyard started with, so a panel
process can take it on its command line from the bar's process.

## Changing it

A change lands here, gets a new tag (`v0.1.1`, or `v0.2.0` when an app has
to change with it), and each app moves its `tag` when it wants the change.
`cargo test`, `cargo clippy --all-targets -- -D warnings` and
`cargo fmt --check` all pass before a tag.

## License

MIT
