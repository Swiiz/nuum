use std::marker::PhantomData;

use nuum_core::{Adapter, Controller, Port};
use nuum_egui::api::util::id_type_map::TypeId;

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

// Only accepts Update<T> or Render<T> dispatching it to the correct ports
struct Multi<Rp, Up, Inner> {
    render_ports: Rp,
    update_ports: Up,
    inner: Inner,
}

pub struct Render<T>(pub T);
pub struct Update<T>(pub T);

impl<Rp, Up, Inner, Event> Controller<Render<Event>> for Multi<Rp, Up, Inner>
where
    Rp: for<'a> Port<'a, Render<Event>, Inner>,
{
    fn run(&mut self, mut event: Render<Event>) {
        self.render_ports.port(&mut event, &mut self.inner);
    }
}

impl<Rp, Up, Inner, Event> Controller<Update<Event>> for Multi<Rp, Up, Inner>
where
    Up: for<'a> Port<'a, Update<Event>, Inner>,
{
    fn run(&mut self, mut event: Update<Event>) {
        self.update_ports.port(&mut event, &mut self.inner);
    }
}
