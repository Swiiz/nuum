use std::{collections::HashMap, sync::Arc};

use nuum::{
    compat::IntoSurfaceTarget,
    core::{platform::Platform, Controller},
    gpu::{surface::GpuSurface, Gpu},
    platform::win::{
        winit::{
            event::WindowEvent,
            window::{WindowAttributes, WindowId},
        },
        WinPlatform, WinPlatformEvent, WinPlatformEventKind,
    },
    render_graph::RenderGraph,
};
use nuum_gpu::wgpu::Color;
use nuum_render_graph::{builtins::SetColorPass, res::RenderGraphAlloc};

fn main() {
    WinPlatform::default().run(&mut App::new());
}

pub struct App {
    gpu: Gpu,
    surfaces: HashMap<WindowId, GpuSurface<'static>>,
    render_graph: RenderGraph,
}

impl App {
    pub fn new() -> Self {
        let gpu = Gpu::new();
        let surfaces = HashMap::new();

        // Render Graph Creation

        let alloc = RenderGraphAlloc::default();
        let view = alloc.frame_view();

        let render_graph = RenderGraph::builder()
            .with_pass("clear", SetColorPass(view.write(), Color::BLACK))
            .build(alloc);

        Self {
            gpu,
            surfaces,
            render_graph,
        }
    }
}

impl<'a> Controller<WinPlatformEvent<'a>> for App {
    fn run(&mut self, input: WinPlatformEvent<'a>) {
        match input.kind {
            WinPlatformEventKind::Init => {
                let win_ptr = input.handle.create_window_ptr(
                    Arc::new,
                    WindowAttributes::default().with_title("nuum example"),
                );

                self.surfaces.insert(
                    win_ptr.id(),
                    self.gpu
                        .acquire_surface(win_ptr.clone().into_surface_target()),
                );
            }
            WinPlatformEventKind::WindowEvent {
                window_id,
                window_event,
            } => match window_event {
                WindowEvent::CloseRequested => {
                    self.surfaces.remove(&window_id);
                    input.handle.remove_window(window_id);
                }
                WindowEvent::RedrawRequested => {
                    let surface = self.surfaces.get_mut(&window_id).unwrap();

                    if let Some(frame) = surface.next_frame(&self.gpu) {
                        let frame = self.render_graph.run(&self.gpu, frame);
                        frame.present(&self.gpu);
                    }

                    input.handle.get_window(window_id).unwrap().request_redraw();
                }
                _ => (),
            },
            _ => (),
        }
    }
}
