# Mouse Recorder GUI (`rustautogui_gui`)

This repository is a standard single-crate Cargo project for the mouse recorder GUI app.

The app uses the published crate from crates.io:

- `rustautogui = { version = "2.5.0", features = ["opencl"] }`

## Highlights

- Timeline-based mouse recording and playback
- Find-target assisted movement (`FindTarget`) before click actions
- YAML save/load for recordings
- Adjustable wait / click speed / move speed controls
- Built with Iced (`0.14`) on top of `rustautogui`

## Project layout

- `src/` — application source
- `assets/` — icon assets
- `Cargo.toml` — crate manifest

## Run

From repository root:

- `cargo run`

## Quick start

1. Run the app.
2. Record a short sequence.
3. Stop recording and edit rows if needed.
4. Save to `recording.yaml`.
5. Replay and iterate.

## Build check

- `cargo check`

## Contributing

- See [`CONTRIBUTING.md`](CONTRIBUTING.md) for repo conventions and commands.

## Upstream crate

For library internals and full API documentation, see upstream:

- [DavorMar/rustautogui](https://github.com/DavorMar/rustautogui)
