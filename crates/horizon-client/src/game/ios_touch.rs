//! Touch on iOS: the engine's on-screen joystick and buttons, and the desktop
//! mouse path for looking, mining, using items and menus.

use std::cell::RefCell;

use winit::event::{
    DeviceEvent, DeviceId, ElementState, MouseButton, Touch, TouchPhase, WindowEvent,
};

use super::*;

/// The desktop pointer the touches outside the overlay drive.
#[derive(Default)]
struct Mouse {
    /// The finger that presses the left button.
    primary: Option<u64>,
    last: (f64, f64),
    fingers: u32,
}

thread_local! {
    static MOUSE: RefCell<Mouse> = RefCell::new(Mouse::default());
}

pub(super) fn handle(
    game: &mut Game<'_>,
    touch: &Touch,
    event_loop: &ActiveEventLoop,
    id: WindowId,
) {
    game.last_input = Instant::now();
    let Some(size) = game.window.as_ref().map(|w| w.inner_size()) else {
        return;
    };
    // A menu is the desktop pointer's: the overlay is not drawn there.
    if game.pad_in_menu() {
        if touch.phase == TouchPhase::Started {
            game.pad.touch.clear();
        }
        return mouse(game, touch, event_loop, id);
    }
    let point = [
        touch.location.x as f32 / size.width.max(1) as f32,
        touch.location.y as f32 / size.height.max(1) as f32,
    ];
    let started = touch.phase == TouchPhase::Started;
    // The joystick and the buttons: the engine's touch layer owns these
    // fingers, and the gamepad path moves, jumps and opens the menus.
    if game.pad.touch.owns_pointer(touch.id)
        || (started && game.pad.touch.control_at_point(point))
    {
        game.pad.in_use = true;
        game.pad.touch.update(
            touch.id,
            point,
            started,
            matches!(touch.phase, TouchPhase::Ended | TouchPhase::Cancelled),
        );
        actions(game);
        return;
    }
    if started {
        // The hotbar row is the world's: a tap selects its slot, never mines.
        let batch = HudBatch::new([size.width, size.height]);
        let p = [
            touch.location.x as f32 / batch.scale,
            touch.location.y as f32 / batch.scale,
        ];
        if let Some(slot) = horizon_client::touch::hotbar_slot(batch.width, batch.height, p) {
            game.selected = slot;
            game.pad.touch.ignore(touch.id);
            return;
        }
    }
    mouse(game, touch, event_loop, id);
}

/// What an overlay button asked for outside the gamepad state.
fn actions(game: &mut Game<'_>) {
    if std::mem::take(&mut game.pad.touch.chat_requested) {
        game.audio.play_button_click();
        game.pad.touch.clear();
        // No on-screen keyboard yet: the chat opens for a hardware keyboard.
        if let Err(error) = game.key_pressed(KeyCode::KeyT) {
            log::warn!("Touch chat: {error}");
        }
    } else if std::mem::take(&mut game.pad.touch.settings_requested) {
        game.audio.play_button_click();
        game.open_options(false);
    } else if std::mem::take(&mut game.pad.touch.perspective_requested) {
        game.audio.play_button_click();
        if let Err(error) = game.key_pressed(KeyCode::F5) {
            log::warn!("Touch perspective: {error}");
        }
    } else if std::mem::take(&mut game.pad.touch.debug_requested) {
        game.audio.play_button_click();
        use horizon_client::debug_keys::KeyAction;
        game.debug_key(KeyCode::F3, KeyAction::Press);
        game.debug_key(KeyCode::F3, KeyAction::Release);
    } else if std::mem::take(&mut game.pad.touch.clicked) {
        game.audio.play_button_click();
    }
}

/// Outside the overlay: drag turns the view, a hold mines or uses, two fingers
/// right-click, and a tap clicks in a menu.
fn mouse(game: &mut Game<'_>, touch: &Touch, event_loop: &ActiveEventLoop, id: WindowId) {
    let (events, motion) = MOUSE.with(|thread| {
        let mut state = thread.borrow_mut();
        let mut events = Vec::new();
        let mut motion = None;
        let device = DeviceId::dummy();
        let position = touch.location;
        match touch.phase {
            TouchPhase::Started => {
                state.fingers += 1;
                if state.fingers == 1 {
                    state.primary = Some(touch.id);
                    state.last = (position.x, position.y);
                    events.push(WindowEvent::CursorMoved {
                        device_id: device,
                        position,
                    });
                    events.push(WindowEvent::MouseInput {
                        device_id: device,
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                    });
                } else if state.fingers == 2 {
                    // A second finger is the right button: use an item, place a block.
                    events.push(WindowEvent::MouseInput {
                        device_id: device,
                        state: ElementState::Released,
                        button: MouseButton::Left,
                    });
                    events.push(WindowEvent::MouseInput {
                        device_id: device,
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
                        device_id: device,
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
                        device_id: device,
                        state: ElementState::Released,
                        button,
                    });
                }
            }
        }
        (events, motion)
    });
    if let Some(delta) = motion {
        game.device_event(
            event_loop,
            DeviceId::dummy(),
            DeviceEvent::MouseMotion { delta },
        );
    }
    for event in events {
        game.window_event(event_loop, id, event);
    }
}
