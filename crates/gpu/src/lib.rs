use surface::{GpuSurface, SurfaceTarget};

pub use wgpu;
pub mod surface;

pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Gpu {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None, //Todo: Add support for webgl
            force_fallback_adapter: false,
        }))
        .unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .unwrap_or_else(|e| panic!("Could not acquire graphics device: {e}"));

        Gpu {
            instance,
            adapter,
            device,
            queue,
        }
    }

    pub fn acquire_surface<'a>(&self, target: impl Into<SurfaceTarget<'a>>) -> GpuSurface<'a> {
        let target = target.into();
        let surface = self
            .instance
            .create_surface(target.target)
            .unwrap_or_else(|e| panic!("Could not create graphics surface: {e}"));
        let capabilities = surface.get_capabilities(&self.adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let mut surface = GpuSurface {
            surface,
            capabilities,
            format,
        };

        surface.resize(self, target.size);
        surface
    }
}
