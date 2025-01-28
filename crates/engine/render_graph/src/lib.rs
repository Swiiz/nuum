use dagga::{Node, Schedule};
use nuum_gpu::{surface::Frame, Gpu};
use pass::{DynPass, PassEncoder, PassNode};
use res::{RenderGraphAlloc, RenderResMap, ResId};

pub mod builtins;
pub mod pass;
pub mod res;

type Dag = dagga::Dag<DynPass, ResId>;

pub struct RenderGraph {
    pub data: RenderResMap,
    schedule: Schedule<PassNode>,
}

impl RenderGraph {
    pub fn builder() -> RenderGraphBuilder {
        RenderGraphBuilder::default()
    }

    pub fn run(&mut self, gpu: &Gpu, frame: Frame) -> Frame {
        let Frame {
            surface_texture,
            mut encoder,
            view,
        } = frame;

        self.data.prepare(view, surface_texture);

        for batch in &mut self.schedule.batches {
            for node in batch {
                (node.inner_mut().run)(&self.data, &mut encoder, gpu);
            }
        }

        let (view, surface_texture) = self.data.finish();

        Frame {
            surface_texture,
            encoder,
            view,
        }
    }
}

#[derive(Default)]
pub struct RenderGraphBuilder {
    dag: Dag,
}

impl RenderGraphBuilder {
    pub fn with_pass(mut self, name: impl Into<String>, pass: impl PassEncoder) -> Self {
        let builder = pass.node_builder();
        let node = Node::new(pass.dyn_pass()).with_name(name);
        self.dag.add_node(builder(node));
        self
    }

    pub fn build(self, alloc: RenderGraphAlloc) -> RenderGraph {
        RenderGraph {
            schedule: self
                .dag
                .build_schedule()
                .unwrap_or_else(|e| panic!("Failed to build render graph: {e}")),
            data: RenderResMap::not_ready(alloc),
        }
    }
}
