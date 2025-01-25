use std::sync::Arc;

use nuum_core::platform::Platform;
use nuum_win_platform::{
    winit::window::WindowAttributes, WinPlatform, WinPlatformEvent, WinPlatformEventKind,
};

fn main() {
    WinPlatform::default().run(&mut AppController);
}

struct AppController;

impl<'a> nuum_core::Controller<WinPlatformEvent<'a>> for AppController {
    fn run(&mut self, input: WinPlatformEvent<'a>) {
        match input.kind {
            WinPlatformEventKind::Init => {
                input.handle.create_window_ptr(
                    Arc::new,
                    WindowAttributes::default().with_title("nuum example"),
                );
            }
            WinPlatformEventKind::WindowEvent {
                window_id,
                window_event,
            } => match window_event {
                winit::event::WindowEvent::CloseRequested => {
                    input.handle.remove_window(window_id);
                }
                _ => (),
            },
            _ => (),
        };
    }
}
