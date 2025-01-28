use crate::RenderEvent;

/// Renderer with platform specific implementation, allowing access to platform event such as input
pub trait NativeRenderer<T, P, Inner> {
    fn on_platform_event(&mut self, input: &mut P);
    fn render_port(&mut self, event: &mut RenderEvent<T>, inner: &mut Inner);
}

impl<T, P, Inner> NativeRenderer<T, P, Inner> for () {
    fn on_platform_event(&mut self, _: &mut P) {}
    fn render_port(&mut self, _: &mut RenderEvent<T>, _: &mut Inner) {}
}
