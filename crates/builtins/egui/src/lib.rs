pub use egui as api;

use egui::{ahash::HashMap, ClippedPrimitive, Context, RawInput, TexturesDelta, ViewportId};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::{
    winit::{
        event::{DeviceEvent, WindowEvent},
        window::WindowId,
    },
    State,
};
use nuum_core::{Controller, Port};
use nuum_gpu::{
    surface::GpuSurface,
    wgpu::{
        rwh::HasDisplayHandle, CommandEncoder, LoadOp, Operations, RenderPassColorAttachment,
        RenderPassDescriptor, StoreOp, TextureView,
    },
    Gpu,
};
use nuum_render_graph::{
    pass::{PassEncoder, PassNode},
    res::{MoveRes, RenderResMap, ResHandle, WriteRes},
};
use nuum_renderer::{native::NativeRenderer, IsRenderEvent, RenderEvent};
use nuum_win_platform::{WinPlatformEvent, WinPlatformEventKind};

pub struct WindowState {
    state: State,
    input: RawInput,
    screen: ScreenDescriptor,
}

#[derive(Default)]
pub struct EguiRenderer {
    states: HashMap<WindowId, WindowState>,
    ctx: Context,
}

impl<T: EguiRenderData, Inner: for<'c, 'd> Controller<EguiRenderEvent<'c, 'd, T>>>
    NativeRenderer<T, WinPlatformEvent<'_>, Inner> for EguiRenderer
{
    fn on_platform_event(&mut self, event: &mut WinPlatformEvent<'_>) {
        match &event.kind {
            WinPlatformEventKind::WindowEvent {
                window_id,
                window_event,
            } => {
                if let Some(window) = event.handle.get_window(*window_id) {
                    let window_state = window_state_lazy(
                        &mut self.states,
                        *window_id,
                        self.ctx.clone(),
                        self.ctx.viewport_id(),
                        window,
                    );
                    let _ = window_state.state.on_window_event(window, window_event);

                    let (w, h): (u32, u32) = window.inner_size().into();
                    if matches!(window_event, WindowEvent::RedrawRequested) && w > 0 && h > 0 {
                        window_state.input = window_state.state.take_egui_input(window);
                        window_state.screen = ScreenDescriptor {
                            size_in_pixels: [w, h],
                            pixels_per_point: window.scale_factor() as f32,
                        };
                    }
                }
            }
            WinPlatformEventKind::DeviceEvent {
                device_id: _,
                device_event,
            } => match device_event {
                DeviceEvent::MouseMotion { delta } => {
                    for window_state in self.states.values_mut() {
                        window_state.state.on_mouse_motion(*delta);
                    }
                }
                _ => (),
            },
            _ => (),
        };
    }

    fn render_port(&mut self, event: &mut RenderEvent<T>, inner: &mut Inner) {
        if let Some(window_state) = self.states.get_mut(&event.window_id()) {
            let full_output = self
                .ctx
                .run(std::mem::take(&mut window_state.input), |ctx| {
                    inner.run(EguiRenderEvent {
                        base: event,
                        egui: ctx.clone(),
                    });
                });
            let paint_jobs = self
                .ctx
                .tessellate(full_output.shapes, full_output.pixels_per_point);

            event
                .access(|res| res.egui_render_payload().result())
                .replace(EguiRenderPayload {
                    screen_descriptor: ScreenDescriptor {
                        size_in_pixels: window_state.screen.size_in_pixels,
                        pixels_per_point: window_state.screen.pixels_per_point,
                    },
                    paint_jobs: paint_jobs.into_boxed_slice(),
                    textures_delta: full_output.textures_delta,
                });
        }
    }
}

pub struct EguiRenderEvent<'a, 'b, T> {
    pub base: &'a mut RenderEvent<'b, T>,
    pub egui: Context,
}

impl<T> IsRenderEvent for EguiRenderEvent<'_, '_, T> {
    type Data = T;
    fn render_data(&self) -> &RenderEvent<T> {
        self.base
    }
}

pub trait EguiRenderData: 'static {
    fn egui_render_payload(&self) -> &ResHandle<EguiRenderPayload>;
}

fn window_state_lazy<'a>(
    states: &'a mut HashMap<WindowId, WindowState>,
    window_id: WindowId,
    ctx: Context,
    viewport: ViewportId,
    display_target: &dyn HasDisplayHandle,
) -> &'a mut WindowState {
    states.entry(window_id).or_insert_with(|| WindowState {
        state: State::new(ctx, viewport, display_target, None, None, None),
        input: RawInput::default(),
        screen: ScreenDescriptor {
            pixels_per_point: 0.,
            size_in_pixels: [0, 0],
        },
    })
}
pub struct EguiRenderPass {
    renderer: Renderer,

    view: WriteRes<TextureView>,
    render_data: MoveRes<EguiRenderPayload>,
}

pub struct EguiRenderPayload {
    screen_descriptor: ScreenDescriptor,
    paint_jobs: Box<[ClippedPrimitive]>,
    textures_delta: TexturesDelta,
}

impl EguiRenderPass {
    pub fn new(
        view: WriteRes<TextureView>,
        render_data: MoveRes<EguiRenderPayload>,
        gpu: &Gpu,
        surface: &GpuSurface<'static>,
    ) -> Self {
        Self {
            view,
            renderer: Renderer::new(&gpu.device, surface.format, None, 1, false),
            render_data,
        }
    }
}

impl PassEncoder for EguiRenderPass {
    fn encode<'a>(&'a mut self, res: &RenderResMap, encoder: &'a mut CommandEncoder, gpu: &Gpu) {
        let Some(render_data) = res.try_access(&self.render_data) else {
            println!("Warn: No egui render data provided, use the EguiRenderer as native renderer in the RenderPort or remove EguiRenderPass from the RenderGraph.");
            return;
        };

        for (id, image_delta) in &render_data.textures_delta.set {
            self.renderer
                .update_texture(&gpu.device, &gpu.queue, *id, image_delta);
        }
        for id in &render_data.textures_delta.free {
            self.renderer.free_texture(id);
        }

        self.renderer.update_buffers(
            &gpu.device,
            &gpu.queue,
            encoder,
            &render_data.paint_jobs,
            &render_data.screen_descriptor,
        );

        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &res.access(&self.view),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let mut render_pass = render_pass.forget_lifetime(); // Renderpass and encoder can't live at the same time?

        // Record all render passes.
        self.renderer.render(
            &mut render_pass,
            &render_data.paint_jobs,
            &render_data.screen_descriptor,
        );
    }

    fn node_builder(&self) -> (impl FnOnce(PassNode) -> PassNode + 'static) {
        let view = self.view.into();
        let render_data = self.render_data.into();

        move |node| node.with_read(view).with_read(render_data)
    }
}
