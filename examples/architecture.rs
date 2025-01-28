use nuum_core::{Adapter, Controller, Port};

pub struct UpdatePort;
pub struct RenderPort;

pub struct Update;
pub struct Render;

impl<'a, I: Controller<Update>> Port<'a, (), I> for UpdatePort {
    fn port(&mut self, _event: &mut (), inner: &mut I) {
        inner.run(Update);
    }
}

impl<'a, I: Controller<Render>> Port<'a, (), I> for RenderPort {
    fn port(&mut self, _event: &mut (), inner: &mut I) {
        inner.run(Render);
    }
}

pub struct World;

impl Controller<Update> for World {
    fn run(&mut self, _event: Update) {}
}

impl Controller<Render> for World {
    fn run(&mut self, _event: Render) {}
}

fn main() {
    let mut app = Adapter {
        ports: (UpdatePort, RenderPort),
        inner: World,
    };

    app.run(());
}
