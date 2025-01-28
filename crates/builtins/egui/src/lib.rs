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
use nuum_renderer::RenderEvent;
use nuum_win_platform::{WinPlatformEvent, WinPlatformEventKind};

pub struct EguiInputPort {
    states: HashMap<WindowId, State>,
    ctx: Context,
}

impl Default for EguiInputPort {
    fn default() -> Self {
        Self {
            states: HashMap::default(),
            ctx: Context::default(),
        }
    }
}

impl<'a, 'b, Inner: Controller<EguiInputEvent>> Port<'a, WinPlatformEvent<'b>, Inner>
    for EguiInputPort
{
    fn port(&mut self, event: &mut WinPlatformEvent, inner: &mut Inner) {
        match &event.kind {
            WinPlatformEventKind::WindowEvent {
                window_id,
                window_event,
            } => {
                if let Some(window) = event.handle.get_window(*window_id) {
                    let _ = state_lazy(
                        &mut self.states,
                        *window_id,
                        self.ctx.clone(),
                        self.ctx.viewport_id(),
                        window,
                    )
                    .on_window_event(window, window_event);

                    let (w, h): (u32, u32) = window.inner_size().into();
                    if matches!(window_event, WindowEvent::RedrawRequested) && w > 0 && h > 0 {
                        let input = self
                            .states
                            .get_mut(window_id)
                            .unwrap()
                            .take_egui_input(window);

                        let (w, h) = window.inner_size().into();
                        inner.run(EguiInputEvent {
                            window_id: *window_id,
                            screen_descriptor: ScreenDescriptor {
                                size_in_pixels: [w, h],
                                pixels_per_point: window.scale_factor() as f32,
                            },
                            input: Some(input),
                            ctx: self.ctx.clone(),
                        });
                    }
                }
            }
            WinPlatformEventKind::DeviceEvent {
                device_id: _,
                device_event,
            } => match device_event {
                DeviceEvent::MouseMotion { delta } => {
                    for state in self.states.values_mut() {
                        state.on_mouse_motion(*delta);
                    }
                }
                _ => (),
            },
            _ => (),
        };
    }
}

pub struct EguiInputEvent {
    pub window_id: WindowId,
    pub screen_descriptor: ScreenDescriptor,
    pub input: Option<RawInput>,
    pub ctx: Context,
}

pub struct EguiRenderPort {
    ctx: Option<Context>,
    render_data: Option<EguiRenderPayload>,
    begin_next: bool,
    submit_current: bool,
}

impl Default for EguiRenderPort {
    fn default() -> Self {
        Self {
            ctx: None,
            render_data: None,
            begin_next: true,
            submit_current: false,
        }
    }
}

pub struct EguiRenderEvent<'a, 'b, T> {
    pub base: &'a mut RenderEvent<'b, T>,
    pub egui: Context,
}

impl<'a, Inner> Port<'a, EguiInputEvent, Inner> for EguiRenderPort {
    fn port(&mut self, event: &mut EguiInputEvent, _: &mut Inner) {
        if self.ctx.is_none() {
            self.ctx = Some(event.ctx.clone());
        }

        if self.submit_current {
            let full_output = event.ctx.end_pass();
            let paint_jobs = event
                .ctx
                .tessellate(full_output.shapes, event.screen_descriptor.pixels_per_point);

            self.render_data.replace(EguiRenderPayload {
                screen_descriptor: ScreenDescriptor {
                    size_in_pixels: event.screen_descriptor.size_in_pixels,
                    pixels_per_point: event.screen_descriptor.pixels_per_point,
                },
                paint_jobs: paint_jobs.into_boxed_slice(),
                textures_delta: full_output.textures_delta,
            });
            self.submit_current = false;
            self.begin_next = true;
        }
        if self.begin_next {
            event.ctx.begin_pass(event.input.take().unwrap());
            self.begin_next = false;
        }
    }
}

pub trait EguiRenderData: 'static {
    fn egui_render_payload(&self) -> &ResHandle<EguiRenderPayload>;
}

impl<'a, 'b, Inner: for<'c, 'd> Controller<EguiRenderEvent<'c, 'd, T>>, T: EguiRenderData>
    Port<'a, RenderEvent<'b, T>, Inner> for EguiRenderPort
{
    fn port(&mut self, event: &'a mut RenderEvent<'b, T>, inner: &mut Inner) {
        if let Some(ctx) = &self.ctx {
            inner.run(EguiRenderEvent {
                base: event,
                egui: ctx.clone(),
            });
            self.submit_current = true;
        }

        if let Some(render_data) = self.render_data.take() {
            event
                .access(|res| res.egui_render_payload().result())
                .replace(render_data);
        }
    }
}

fn state_lazy<'a>(
    states: &'a mut HashMap<WindowId, State>,
    window_id: WindowId,
    ctx: Context,
    viewport: ViewportId,
    display_target: &dyn HasDisplayHandle,
) -> &'a mut State {
    states
        .entry(window_id)
        .or_insert_with(|| State::new(ctx, viewport, display_target, None, None, None))
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
