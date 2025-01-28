use crate::Controller;

pub trait Platform {
    type Event<'a>: PlatformEvent;

    fn run<T: for<'a> Controller<Self::Event<'a>>>(&mut self, controller: &mut T);
}

pub trait PlatformEvent {
    fn exit(&self);
    fn is_update(&self) -> bool;
}
