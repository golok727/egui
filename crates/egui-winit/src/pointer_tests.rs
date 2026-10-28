use super::*;
use winit::event::{
    ButtonSource, FingerId, Force, PointerKind, PointerSource, TabletToolButton, TabletToolData,
    TabletToolKind, WindowEvent,
};

struct NoDisplay;

impl HasDisplayHandle for NoDisplay {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::Unavailable)
    }
}

fn state() -> State {
    State::new(
        egui::Context::default(),
        ViewportId::ROOT,
        &NoDisplay,
        Some(2.0),
        None,
        None,
    )
}

fn pen_data(force: f64) -> TabletToolData {
    TabletToolData {
        force: Some(Force::Normalized(force)),
        ..Default::default()
    }
}

fn button(state: ElementState, button: ButtonSource, primary: bool, x: f64) -> WindowEvent {
    WindowEvent::PointerButton {
        device_id: None,
        state,
        position: PhysicalPosition::new(x, 40.0),
        primary,
        button,
        is_macos_activation_click: false,
    }
}

fn pen_button(state: ElementState, force: f64, x: f64) -> WindowEvent {
    button(
        state,
        ButtonSource::TabletTool {
            kind: TabletToolKind::Pen,
            button: TabletToolButton::Contact,
            data: pen_data(force),
        },
        true,
        x,
    )
}

#[test]
fn pen_stroke_preserves_pressure_and_release_position() {
    let mut state = state();
    let _ = state.on_pointer_event(&pen_button(ElementState::Pressed, 0.25, 20.0), 2.0);
    let _ = state.on_pointer_event(
        &WindowEvent::PointerMoved {
            device_id: None,
            position: PhysicalPosition::new(30.0, 40.0),
            primary: true,
            source: PointerSource::TabletTool {
                kind: TabletToolKind::Pen,
                data: pen_data(0.75),
            },
        },
        2.0,
    );
    let _ = state.on_pointer_event(&pen_button(ElementState::Released, 0.0, 50.0), 2.0);

    let samples: Vec<_> = state
        .egui_input
        .events
        .iter()
        .filter_map(|event| match event {
            egui::Event::Touch {
                phase, pos, force, ..
            } => Some((*phase, *pos, *force)),
            _ => None,
        })
        .collect();
    assert_eq!(
        samples,
        [
            (egui::TouchPhase::Start, egui::pos2(10.0, 20.0), Some(0.25)),
            (egui::TouchPhase::Move, egui::pos2(15.0, 20.0), Some(0.75)),
            (egui::TouchPhase::End, egui::pos2(25.0, 20.0), Some(0.0)),
        ]
    );
    assert!(
        state
            .egui_input
            .events
            .contains(&egui::Event::PointerButton {
                pos: egui::pos2(25.0, 20.0),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            })
    );
    assert!(!state.is_any_pointer_button_down());
}

#[test]
fn pen_hover_does_not_start_a_touch() {
    let mut state = state();
    let _ = state.on_pointer_event(
        &WindowEvent::PointerMoved {
            device_id: None,
            position: PhysicalPosition::new(20.0, 40.0),
            primary: true,
            source: PointerSource::TabletTool {
                kind: TabletToolKind::Pen,
                data: pen_data(0.0),
            },
        },
        2.0,
    );
    assert_eq!(
        state.egui_input.events,
        [egui::Event::PointerMoved(egui::pos2(10.0, 20.0))]
    );
}

#[test]
fn leaving_during_pen_contact_cancels_and_releases() {
    let mut state = state();
    let _ = state.on_pointer_event(&pen_button(ElementState::Pressed, 0.5, 20.0), 2.0);
    state.egui_input.events.clear();
    let _ = state.on_pointer_event(
        &WindowEvent::PointerLeft {
            device_id: None,
            position: None,
            primary: true,
            kind: PointerKind::TabletTool(TabletToolKind::Pen),
        },
        2.0,
    );
    assert!(matches!(
        state.egui_input.events[0],
        egui::Event::Touch {
            phase: egui::TouchPhase::Cancel,
            ..
        }
    ));
    assert!(matches!(
        state.egui_input.events[1],
        egui::Event::PointerButton { pressed: false, .. }
    ));
    assert_eq!(state.egui_input.events[2], egui::Event::PointerGone);
    assert!(!state.is_any_pointer_button_down());
}

#[test]
fn secondary_finger_keeps_pressure_without_moving_the_pointer() {
    let mut state = state();
    let first = FingerId::from_raw(1);
    let second = FingerId::from_raw(2);
    for (finger_id, primary, x) in [(first, true, 20.0), (second, false, 80.0)] {
        let _ = state.on_pointer_event(
            &button(
                ElementState::Pressed,
                ButtonSource::Touch {
                    finger_id,
                    force: None,
                },
                primary,
                x,
            ),
            2.0,
        );
    }
    state.egui_input.events.clear();
    let _ = state.on_pointer_event(
        &WindowEvent::PointerMoved {
            device_id: None,
            position: PhysicalPosition::new(90.0, 40.0),
            primary: false,
            source: PointerSource::Touch {
                finger_id: second,
                force: Some(Force::Calibrated {
                    force: 2.0,
                    max_possible_force: 4.0,
                }),
            },
        },
        2.0,
    );
    assert!(matches!(
        state.egui_input.events.as_slice(),
        [egui::Event::Touch {
            phase: egui::TouchPhase::Move,
            force: Some(0.5),
            ..
        }]
    ));
    assert_eq!(state.pointer_pos_in_points, Some(egui::pos2(10.0, 20.0)));

    let _ = state.on_pointer_event(
        &WindowEvent::PointerLeft {
            device_id: None,
            position: None,
            primary: false,
            kind: PointerKind::Touch(second),
        },
        2.0,
    );
    assert!(matches!(
        state.egui_input.events.last(),
        Some(egui::Event::Touch {
            phase: egui::TouchPhase::Cancel,
            ..
        })
    ));
}

#[test]
fn mouse_click_uses_its_own_position() {
    let mut state = state();
    for (state_, mouse_button, x) in [
        (ElementState::Pressed, winit::event::MouseButton::Left, 20.0),
        (
            ElementState::Pressed,
            winit::event::MouseButton::Right,
            30.0,
        ),
        (
            ElementState::Released,
            winit::event::MouseButton::Left,
            40.0,
        ),
    ] {
        let _ = state.on_pointer_event(
            &button(state_, ButtonSource::Mouse(mouse_button), true, x),
            2.0,
        );
    }
    assert_eq!(state.pointer_pos_in_points, Some(egui::pos2(20.0, 20.0)));
    let _ = state.on_pointer_event(
        &button(
            ElementState::Released,
            ButtonSource::Mouse(winit::event::MouseButton::Right),
            true,
            50.0,
        ),
        2.0,
    );
    assert!(!state.is_any_pointer_button_down());
    assert!(
        !state
            .egui_input
            .events
            .iter()
            .any(|event| matches!(event, egui::Event::Touch { .. }))
    );
}
