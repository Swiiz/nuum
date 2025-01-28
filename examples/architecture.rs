use nuum_core::{
    Adapter, Controller, Port,
};

pub struct UpdatePort;
pub struct RenderPort;

pub struct WorldUpdateEvent;
pub struct WorldRenderEvent;

impl<'a, I: Controller<WorldUpdateEvent>> Port<'a, (), I> for UpdatePort {
    fn port(&mut self, _event: &mut (), inner: &mut I) {
        inner.run(WorldUpdateEvent);
    }
}

impl<'a, I: Controller<WorldRenderEvent>> Port<'a, (), I> for RenderPort {
    fn port(&mut self, _event: &mut (), inner: &mut I) {
        inner.run(WorldRenderEvent);
    }
}

pub struct World;

impl Controller<WorldUpdateEvent> for World {
    fn run(&mut self, _event: WorldUpdateEvent) {}
}

impl Controller<WorldRenderEvent> for World {
    fn run(&mut self, _event: WorldRenderEvent) {}
}

fn main() {
    let mut app = Adapter {
        ports: (UpdatePort, RenderPort),
        inner: World,
    };

    app.run(());
}

//////////////////////////////////////////////
