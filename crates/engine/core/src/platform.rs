use crate::Controller;

pub trait Platform {
    type PlatformHandle<'a>: PlatformHandle;
    type Output<'a>: AsMut<Self::PlatformHandle<'a>>;

    fn run<T: for<'a> Controller<Self::Output<'a>>>(&mut self, controller: &mut T);
}

pub trait PlatformHandle {
    fn exit(&self);
}
