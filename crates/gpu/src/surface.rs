use nuum_core::maths::Vector2;

use crate::Gpu;

pub use wgpu::SurfaceTarget as RawSurfaceTarget;
pub struct SurfaceTarget<'a> {
    pub size: Vector2<u32>,
    pub target: RawSurfaceTarget<'a>,
}

pub struct GpuSurface<'a> {
    pub surface: wgpu::Surface<'a>,
    pub capabilities: wgpu::SurfaceCapabilities,
    pub format: wgpu::TextureFormat,
}

impl GpuSurface<'_> {
    pub fn resize(&mut self, gpu: &Gpu, size: impl Into<Vector2<u32>>) {
        let size = size.into();
        self.surface.configure(
            &gpu.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                width: size.x,
                height: size.y,
                present_mode: self.capabilities.present_modes[0],
                alpha_mode: self.capabilities.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        );
    }

    pub fn next_frame<'a>(&'a mut self, gpu: &Gpu) -> Option<Frame> {
        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(|e| match e {
                wgpu::SurfaceError::OutOfMemory => {
                    panic!("The system is out of memory for rendering!")
                }
                _ => format!("An error occured during surface texture acquisition: {e}"),
            })
            .ok()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        Some(Frame {
            surface_texture,
            encoder,
            view,
        })
    }
}

pub struct Frame {
    pub view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
    pub surface_texture: wgpu::SurfaceTexture,
}

impl Frame {
    pub fn present(self, gpu: &Gpu) {
        gpu.queue.submit(std::iter::once(self.encoder.finish()));
        self.surface_texture.present();
    }
}
