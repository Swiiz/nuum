use std::any::type_name;

use nuum_gpu::{
    wgpu::{self, TextureView},
    Gpu,
};

use crate::{
    pass::{Pass, PassNode},
    res::{RenderResMap, WriteRes},
};

pub struct SetColorPass(pub WriteRes<TextureView>, pub wgpu::Color);

impl Pass for SetColorPass {
    fn encode(&self, res: &RenderResMap, encoder: &mut wgpu::CommandEncoder, _: &Gpu) {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(type_name::<Self>()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &res.access(&self.0),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.1),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }

    fn node_builder(&self) -> (impl FnOnce(PassNode) -> PassNode + 'static) {
        let view = self.0.clone().into();

        move |node| node.with_write(view)
    }
}
