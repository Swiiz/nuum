use std::sync::Arc;

use nuum_core::Port;
use winit::{event::WindowEvent, window::WindowAttributes};

use crate::{WinPlatformEvent, WinPlatformEventKind};
pub struct SingleWindowPort(pub WindowAttributes);

impl Default for SingleWindowPort {
    fn default() -> Self {
        Self(WindowAttributes::default().with_title("Nuum window"))
    }
}

impl<'a, 'b, Inner> Port<'a, WinPlatformEvent<'b>, Inner> for SingleWindowPort {
    fn port(&mut self, event: &mut WinPlatformEvent, _: &mut Inner) {
        match &event.kind {
            WinPlatformEventKind::Init => {
                event
                    .handle
                    .create_window_ptr(Arc::new, std::mem::take(&mut self.0));
            }
            WinPlatformEventKind::WindowEvent {
                window_id,
                window_event,
            } => match window_event {
                WindowEvent::CloseRequested => {
                    event.handle.remove_window(*window_id);
                }
                _ => (),
            },

            _ => (),
        }
    }
}
