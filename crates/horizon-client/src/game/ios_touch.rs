//! Touch as a mouse on iOS: drag to look, tap to use, two fingers to right-click.
//! The on-screen buttons, joystick and keyboard the Android build draws are not
//! ported yet, so the desktop controls (crosshair, tap to mine, tap to place)
//! do the work.

use std::cell::RefCell;

use winit::event::{DeviceId, ElementState, MouseButton, Touch, TouchPhase, WindowEvent};

#[derive(Default)]
struct State {
    /// The finger that presses the left button.
    primary: Option<u64>,
    last: (f64, f64),
    fingers: u32,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// The window events a touch becomes, and the pointer motion for the view.
/// The game's own mouse handling does the rest: the same path a desktop mouse
/// takes, so held mining, block placing and every menu work unchanged.
pub(super) fn translate(touch: &Touch) -> (Vec<WindowEvent>, Option<(f64, f64)>) {
    let mut events = Vec::new();
    let mut motion = None;
    let id = DeviceId::dummy();
    let mut state = STATE.with(|s| s.borrow_mut());
    let position = touch.location;
    match touch.phase {
        TouchPhase::Started => {
            state.fingers += 1;
            if state.fingers == 1 {
                state.primary = Some(touch.id);
                state.last = (position.x, position.y);
                events.push(WindowEvent::CursorMoved {
                    device_id: id,
                    position,
                });
                events.push(WindowEvent::MouseInput {
                    device_id: id,
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                });
            } else if state.fingers == 2 {
                // A second finger is the right button: use an item, place a block.
                events.push(WindowEvent::MouseInput {
                    device_id: id,
                    state: ElementState::Released,
                    button: MouseButton::Left,
                });
                events.push(WindowEvent::MouseInput {
                    device_id: id,
                    state: ElementState::Pressed,
                    button: MouseButton::Right,
                });
            }
        }
        TouchPhase::Moved => {
            if state.primary == Some(touch.id) && state.fingers == 1 {
                let delta = (position.x - state.last.0, position.y - state.last.1);
                state.last = (position.x, position.y);
                events.push(WindowEvent::CursorMoved {
                    device_id: id,
                    position,
                });
                motion = Some(delta);
            }
        }
        TouchPhase::Ended | TouchPhase::Cancelled => {
            state.fingers = state.fingers.saturating_sub(1);
            if state.primary == Some(touch.id) {
                state.primary = None;
                let button = if state.fingers > 0 {
                    MouseButton::Right
                } else {
                    MouseButton::Left
                };
                events.push(WindowEvent::MouseInput {
                    device_id: id,
                    state: ElementState::Released,
                    button,
                });
            }
        }
    }
    (events, motion)
}
