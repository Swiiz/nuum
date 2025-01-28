pub use nuum_core as core;

pub mod platform {
    #[cfg(feature = "win_platform")]
    pub use nuum_win_platform as win;
}

#[cfg(feature = "gpu")]
pub use nuum_gpu as gpu;
#[cfg(feature = "renderer")]
pub use nuum_render_graph as render_graph;

/*
pub mod compat {
    use std::borrow::Borrow;

    #[cfg(feature = "gpu")]
    pub trait IntoSurfaceTarget<'a>: 'a {
        fn into_surface_target(self) -> super::gpu::surface::SurfaceTarget<'a>;
    }

    #[cfg(all(feature = "win_platform", feature = "gpu"))]
    impl<
            'a,
            T: Borrow<super::platform::win::winit::window::Window>
                + Into<super::gpu::surface::RawSurfaceTarget<'a>>
                + 'a,
        > IntoSurfaceTarget<'a> for T
    {
        fn into_surface_target(self) -> super::gpu::surface::SurfaceTarget<'a> {
            let win = self.borrow();
            let (w, h) = win.inner_size().into();
            super::gpu::surface::SurfaceTarget {
                size: [w, h].into(),
                target: Into::<_>::into(self),
            }
        }
    }
}
 */
