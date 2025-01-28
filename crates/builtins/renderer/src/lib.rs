use std::{collections::HashMap, sync::Arc};

use nuum_core::{Controller, Port};
use nuum_gpu::{
    surface::{GpuSurface, SurfaceTarget},
    Gpu,
};
use nuum_render_graph::{res::ResAccessor, RenderGraph};
use nuum_win_platform::{
    winit::{
        event::WindowEvent,
        window::{Window, WindowId},
    },
    WinPlatformEvent, WinPlatformEventKind, WinPlatformHandle,
};

pub type SurfaceRenderers<T> = HashMap<WindowId, SurfaceRenderer<T>>;

pub struct RenderPort<T> {
    gpu: Gpu,
    builder: Box<dyn FnMut(&Gpu, &GpuSurface<'static>) -> (RenderGraph, T)>,
    surfaces: SurfaceRenderers<T>,
}

pub struct SurfaceRenderer<T> {
    surface: GpuSurface<'static>,
    render_graph: RenderGraph,
    res: T,

    should_close: bool,
}

impl<T> RenderPort<T> {
    pub fn new(
        builder: impl FnMut(&Gpu, &GpuSurface<'static>) -> (RenderGraph, T) + 'static,
    ) -> Self {
        let gpu = Gpu::new();
        let surfaces = HashMap::new();
        let builder = Box::new(builder);

        Self {
            gpu,
            builder,
            surfaces,
        }
    }
}

pub struct RenderEvent<'a, T> {
    window_id: WindowId,
    surface_renderer: &'a mut SurfaceRenderer<T>,
}

impl<'a, T> RenderEvent<'a, T> {
    pub fn access<U: ResAccessor>(&'a self, f: impl FnOnce(&T) -> U) -> U::Value<'a> {
        let u = f(&self.surface_renderer.res);
        self.surface_renderer.render_graph.data.access(&u)
    }
}

impl<'a, T> RenderEvent<'a, T> {
    pub fn window_id(&self) -> WindowId {
        self.window_id
    }

    pub fn get_surface(&self) -> &GpuSurface<'static> {
        &self.surface_renderer.surface
    }
}

impl<'a, 'b, T, Inner: for<'c> Controller<RenderEvent<'c, T>>> Port<'a, WinPlatformEvent<'b>, Inner>
    for RenderPort<T>
{
    fn port(&mut self, input: &mut WinPlatformEvent, inner: &mut Inner) {
        match &input.kind {
            WinPlatformEventKind::WindowEvent {
                window_id,
                window_event,
            } => match window_event {
                WindowEvent::RedrawRequested => {
                    let Some(surface_renderer) = surface_renderer_lazy(
                        &mut self.surfaces,
                        &mut self.builder,
                        &self.gpu,
                        *window_id,
                        input.handle,
                    ) else {
                        return;
                    };

                    if let Some(frame) = surface_renderer.surface.next_frame(&self.gpu) {
                        surface_renderer
                            .render_graph
                            .run(&self.gpu, frame)
                            .present(&self.gpu);
                    }

                    if let Some(window) = input.handle.get_window(*window_id) {
                        window.request_redraw();

                        inner.run(RenderEvent {
                            window_id: *window_id,
                            surface_renderer,
                        });
                    } else {
                        surface_renderer.should_close = true;
                    }
                }
                WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                    if let Some(surface_renderer) = self.surfaces.get_mut(&window_id) {
                        let (w, h): (u32, u32) = input
                            .handle
                            .get_window(*window_id)
                            .unwrap()
                            .inner_size()
                            .into();
                        surface_renderer.surface.resize(&self.gpu, [w, h]);
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

fn surface_renderer_lazy<'a, T>(
    surfaces: &'a mut SurfaceRenderers<T>,
    builder: &mut Box<dyn FnMut(&Gpu, &GpuSurface<'static>) -> (RenderGraph, T)>,
    gpu: &Gpu,
    window_id: WindowId,
    handle: &mut WinPlatformHandle,
) -> Option<&'a mut SurfaceRenderer<T>> {
    if let Some(
        __ @ SurfaceRenderer {
            should_close: true, ..
        },
    ) = surfaces.get(&window_id)
    {
        surfaces.remove(&window_id);
        return None;
    }

    Some(surfaces.entry(window_id).or_insert_with(|| {
            let window_ptr = handle.get_window_ptr::<Arc<Window>>(window_id).expect(
                "Failed to get window pointer: RenderPort needs Arc<_> window ptr to acquire surface with static lifetime",
            );
            let surface = gpu
                .acquire_surface(SurfaceTarget {
                    size: {
                        let (w, h) = window_ptr.inner_size().into();
                        [w, h].into()
                    },
                    target: window_ptr.clone().into()
                });
                let (render_graph, res) = builder(gpu, &surface);

            SurfaceRenderer {surface, render_graph, res, should_close: false}
        }))
}
