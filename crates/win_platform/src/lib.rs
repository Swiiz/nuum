use internals::{WindowPtr, WindowVec};
use nuum_core::platform::{Platform, PlatformHandle};
use winit::{
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

mod internals;

pub use winit;

pub struct WinPlatformEvent<'a> {
    pub handle: &'a mut WinPlatformHandle<'a>,
    pub kind: WinPlatformEventKind,
}

pub enum WinPlatformEventKind {
    Init,
    WindowEvent {
        window_id: WindowId,
        window_event: WindowEvent,
    },
    DeviceEvent {
        device_id: DeviceId,
        device_event: DeviceEvent,
    },
    AboutToWait,
}

pub struct WinPlatform {
    pub control_flow: ControlFlow,
}

pub struct WinPlatformHandle<'a> {
    event_loop: &'a ActiveEventLoop,
    windows: &'a mut WindowVec,
}

impl WinPlatformHandle<'_> {
    pub fn create_window(&mut self, attrs: WindowAttributes) -> &Window {
        self.create_window_ptr(|w| w, attrs)
    }

    pub fn create_window_ptr<T: WindowPtr>(
        &mut self,
        builder: impl FnOnce(Window) -> T,
        attrs: WindowAttributes,
    ) -> &T {
        let window = self
            .event_loop
            .create_window(attrs)
            .expect("Window creation failed");
        let id = window.id();
        self.windows.push(Box::new(builder(window)));
        self.get_window_ptr(id).expect("Window creation failed")
    }

    pub fn get_window(&mut self, id: WindowId) -> Option<&Window> {
        self.get_dyn_window_ptr(id).map(|w| w.borrow())
    }

    /// If you are using a referenced counted pointer, this will not drop the window and you will need to drop every instance manually or only use `Weak`.
    pub fn remove_window(&mut self, id: WindowId) -> Option<Box<dyn WindowPtr>> {
        Some(
            self.windows.swap_remove(
                self.windows
                    .iter()
                    .position(|w| w.as_ref().borrow().id() == id)?,
            ),
        )
    }

    pub fn get_window_ptr<T: WindowPtr>(&mut self, id: WindowId) -> Option<&T> {
        self.get_dyn_window_ptr(id)
            .map(|w| w.as_any_ref().downcast_ref::<T>().expect("Failed to downcast window pointer, the specified window pointer type doesn't match the actual window ptr type"))
    }

    pub fn get_dyn_window_ptr(&mut self, id: WindowId) -> Option<&dyn WindowPtr> {
        self.windows
            .iter()
            .find_map(|w| (w.as_ref().borrow().id() == id).then_some(w.as_ref()))
    }
}

impl PlatformHandle for WinPlatformHandle<'_> {
    fn exit(&self) {
        self.event_loop.exit();
    }
}

impl<'a> AsMut<WinPlatformHandle<'a>> for WinPlatformEvent<'a> {
    fn as_mut(&mut self) -> &mut WinPlatformHandle<'a> {
        self.handle
    }
}

impl Default for WinPlatform {
    fn default() -> Self {
        Self {
            control_flow: ControlFlow::Poll,
        }
    }
}

impl Platform for WinPlatform {
    type PlatformHandle<'a> = WinPlatformHandle<'a>;
    type Output<'a> = WinPlatformEvent<'a>;

    fn run<T: for<'a> nuum_core::Controller<Self::Output<'a>>>(&mut self, controller: &mut T) {
        let event_loop = EventLoop::new().unwrap();
        let mut runner = internals::WinPlatformRunner::<T>::new(controller);

        event_loop.set_control_flow(self.control_flow);

        event_loop.run_app(&mut runner).unwrap();
    }
}
