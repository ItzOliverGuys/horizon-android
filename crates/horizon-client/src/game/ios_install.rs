//! First launch on iOS: download and prepare the vanilla resources on the device,
//! with the same installer the Android build runs, then quit so the next launch
//! starts the game. There is no progress screen yet: the window stays blank while
//! the log records each stage.

use super::*;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct Installing {
    window: Option<Arc<Window>>,
    done: Arc<AtomicBool>,
    error: Arc<Mutex<Option<String>>>,
}

impl ApplicationHandler for Installing {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        match event_loop.create_window(Window::default_attributes().with_title("Horizon")) {
            Ok(window) => self.window = Some(Arc::new(window)),
            Err(error) => {
                log::error!("Horizon install: no window: {error}");
                std::process::exit(1);
            }
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|w| w.id() != id) {
            return;
        }
        if matches!(event, WindowEvent::CloseRequested) {
            std::process::exit(1);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(200),
        ));
        if !self.done.load(Ordering::Acquire) {
            return;
        }
        // The app is restarted to enter the game: one winit event loop runs per
        // process on iOS (UIApplicationMain never returns).
        match self.error.lock().ok().and_then(|mut e| e.take()) {
            Some(error) => {
                log::error!("Horizon install failed: {error}");
                std::process::exit(1);
            }
            None => std::process::exit(0),
        }
    }
}

/// Downloads the resources into `home` and exits the process: 0 when the game can
/// start next time, 1 when the install failed (the cache stays for a retry).
pub(super) fn show(home: &std::path::Path) -> Result<()> {
    // The generated engine tables ride in the app bundle; the installer unpacks
    // them from the game folder.
    let packed = std::env::current_exe()?.with_file_name("engine-data.zip");
    std::fs::copy(&packed, home.join("engine-data.zip"))
        .map_err(|error| format!("Cannot copy {}: {error}", packed.display()))?;
    let done = Arc::new(AtomicBool::new(false));
    let error = Arc::new(Mutex::new(None));
    let home = home.to_owned();
    let (thread_done, thread_error) = (done.clone(), error.clone());
    std::thread::spawn(move || {
        let result = horizon_client::official_install::install_with_progress(
            &home,
            |progress| {
                log::info!(
                    "Horizon install: {} ({}/{})",
                    progress.stage.label(),
                    progress.done,
                    progress.total
                );
            },
        );
        if let Err(error) = result {
            if let Ok(mut slot) = thread_error.lock() {
                *slot = Some(error.to_string());
            }
        }
        thread_done.store(true, Ordering::Release);
    });
    let mut app = Installing {
        window: None,
        done,
        error,
    };
    EventLoop::new()?.run_app(&mut app)?;
    Ok(())
}
