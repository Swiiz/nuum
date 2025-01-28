use std::time::Instant;

use nuum::{
    core::{platform::Platform, Adapter, Controller},
    gpu::{surface::GpuSurface, wgpu::Color, Gpu},
    platform::win::builtins::SingleWindowPort,
    platform::win::WinPlatform,
    render_graph::{
        builtins::SetColorPass,
        pass::PassScheduler,
        res::{RenderGraphAlloc, ResHandle},
        RenderGraph,
    },
};

use nuum_egui::{EguiRenderData, EguiRenderEvent, EguiRenderPass, EguiRenderPayload, EguiRenderer};
use nuum_renderer::{IsRenderEvent, RenderEvent, RenderPort};

fn main() {
    let mut app = Adapter {
        ports: (
            SingleWindowPort::default(),
            RenderPort::new_with_native(render_graph, EguiRenderer::default()),
        ),
        inner: App::default(),
    };

    WinPlatform::default().run(&mut app);
}

struct App {
    start: Instant,
    background_color: [f32; 3],
}

impl Default for App {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            background_color: [0.0, 0.0, 0.0],
        }
    }
}

impl<'a, 'b> Controller<EguiRenderEvent<'a, 'b, RenderData>> for App {
    fn run(&mut self, mut event: EguiRenderEvent<'a, 'b, RenderData>) {
        nuum_egui::api::Window::new("Nuum EGUI window").show(&mut event.egui, |ui| {
            ui.heading(format!("Hello world! {}", self.start.elapsed().as_millis()));
            ui.separator();
            ui.label("Background color:");
            ui.color_edit_button_rgb(&mut self.background_color);
        });

        let mut render_bg_color = event.render_data().access(|data| data.background.write());
        render_bg_color.r = self.background_color[0] as f64;
        render_bg_color.g = self.background_color[1] as f64;
        render_bg_color.b = self.background_color[2] as f64;
    }
}

impl<'a> Controller<RenderEvent<'a, RenderData>> for App {
    fn run(&mut self, _: RenderEvent<'a, RenderData>) {}
}

pub struct RenderData {
    egui: ResHandle<EguiRenderPayload>,
    background: ResHandle<Color>,
}

impl EguiRenderData for RenderData {
    fn egui_render_payload(&self) -> &ResHandle<EguiRenderPayload> {
        &self.egui
    }
}

pub fn render_graph(gpu: &Gpu, surface: &GpuSurface<'static>) -> (RenderGraph, RenderData) {
    let mut alloc = RenderGraphAlloc::default();
    let data = RenderData {
        egui: alloc.push(None),
        background: alloc.push(Some(Color::BLACK)),
    };

    let view = alloc.frame_view();
    let render_graph = RenderGraph::builder()
        .with_pass("clear", SetColorPass(view.write(), data.background.read()))
        .with_pass(
            "egui",
            EguiRenderPass::new(view.write(), data.egui.move_(), gpu, surface).run_after("clear"),
        )
        .build(alloc);

    (render_graph, data)
}
