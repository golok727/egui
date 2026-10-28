# egui-winit

[![Latest version](https://img.shields.io/crates/v/egui-winit.svg)](https://crates.io/crates/egui-winit)
[![Documentation](https://docs.rs/egui-winit/badge.svg)](https://docs.rs/egui-winit)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

This crates provides bindings between [`egui`](https://github.com/emilk/egui) and [`winit`](https://crates.io/crates/winit).

The library translates winit events to egui, handled copy/paste, updates the cursor, open links clicked in egui, etc.

## Winit 0.31 beta

This branch pins `egui-winit` to `winit = "=0.31.0-beta.4"`.
Windows and event loops are now traits. Pass `window.as_ref()` from a
`Box<dyn winit::window::Window>` or `Arc<dyn winit::window::Window>` to `State`.
Use `surface_size()` and `WindowEvent::SurfaceResized` in the application.

Pen contact emits `egui::Event::Touch` with normalized `force`, alongside the
usual egui pointer events. Hover only moves the pointer. Read stroke samples
from `RawInput::events` before passing the input to egui, or from
`ctx.input(|input| input.events.clone())`:

```rust,ignore
for event in &raw_input.events {
    if let egui::Event::Touch { phase, pos, force, .. } = event {
        // `force` is None when the device does not report pressure.
        paint_sample(*phase, *pos, *force);
    }
}
```

For tilt, rotation, tool kind, and barrel buttons, inspect winit's
`PointerSource::TabletTool` and `ButtonSource::TabletTool` in the application's
window event handler. The existing egui input API carries pressure but does not
carry those additional tablet properties. `Event::Touch` also includes fingers;
use the winit source when the canvas needs to distinguish fingers from a pen.

Call `State::on_window_event_with_event_loop` to enable native file drops.
Winit 0.31 requires the active event loop to accept drops and fetch file URIs.
The original `on_window_event` remains available for other input.

The released AccessKit adapter requires winit 0.30, so disable the `accesskit`
feature on this branch. The workspace's `eframe`, `egui_glow`, and optional
`egui-wgpu/winit` integrations still target winit 0.30. Use a custom integration
with `egui`, `egui-winit`, and the renderer in `egui-wgpu` without its `winit`
feature. All three egui crates must come from the same fork revision.
