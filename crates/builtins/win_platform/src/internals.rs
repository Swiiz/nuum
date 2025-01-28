use nuum_core::{platform::PlatformHandle, Controller};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

use crate::{WinPlatformEvent, WinPlatformEventKind, WinPlatformHandle, WindowPtr};

pub type WindowVec = Vec<Box<dyn WindowPtr>>;

pub struct WinPlatformRunner<'a, T: for<'b> Controller<WinPlatformEvent<'b>>> {
    controller: &'a mut T,
    pub(super) windows: WindowVec,
}

impl<'a, T: for<'b> Controller<WinPlatformEvent<'b>>> WinPlatformRunner<'a, T> {
    pub fn new(controller: &'a mut T) -> Self {
        Self {
            controller,
            windows: Vec::new(),
        }
    }
}

impl<'a, T: for<'b> Controller<WinPlatformEvent<'b>>> ApplicationHandler
    for WinPlatformRunner<'a, T>
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut handle = WinPlatformHandle {
            event_loop,
            windows: &mut self.windows,
        };

        self.controller.run(WinPlatformEvent {
            handle: &mut handle,
            kind: WinPlatformEventKind::Init,
        });
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        window_event: WindowEvent,
    ) {
        let mut handle = WinPlatformHandle {
            event_loop,
            windows: &mut self.windows,
        };

        self.controller.run(WinPlatformEvent {
            handle: &mut handle,
            kind: WinPlatformEventKind::WindowEvent {
                window_id,
                window_event,
            },
        });
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        device_event: winit::event::DeviceEvent,
    ) {
        let mut handle = WinPlatformHandle {
            event_loop,
            windows: &mut self.windows,
        };

        self.controller.run(WinPlatformEvent {
            handle: &mut handle,
            kind: WinPlatformEventKind::DeviceEvent {
                device_id,
                device_event,
            },
        });
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut handle = WinPlatformHandle {
            event_loop,
            windows: &mut self.windows,
        };

        // Close the application if there are no windows
        if handle.windows.is_empty() {
            handle.exit();
        }

        self.controller.run(WinPlatformEvent {
            handle: &mut handle,
            kind: WinPlatformEventKind::AboutToWait,
        });
    }
}
