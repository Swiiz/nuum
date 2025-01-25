use std::borrow::Cow;

use dagga::Node;
use nuum_gpu::{wgpu::CommandEncoder, Gpu};

use crate::res::{RenderResMap, ResId};

pub type PassNode = Node<DynPass, ResId>;

pub trait Pass: Sized + 'static {
    fn encode(&self, res: &RenderResMap, encoder: &mut CommandEncoder, gpu: &Gpu);
    fn node_builder(&self) -> (impl FnOnce(PassNode) -> PassNode + 'static);

    fn dyn_pass(self) -> DynPass {
        DynPass {
            run: Box::new(move |res, enc, gpu| self.encode(res, enc, gpu)),
        }
    }
}

pub struct DynPass {
    pub run: Box<dyn Fn(&RenderResMap, &mut CommandEncoder, &Gpu)>,
}

pub trait PassScheduler: Pass {
    fn run_before(self, name: impl Into<Cow<'static, str>>) -> RunBefore<Self>;
    fn run_after(self, name: impl Into<Cow<'static, str>>) -> RunAfter<Self>;
}

impl<T: Pass> PassScheduler for T {
    fn run_before(self, name: impl Into<Cow<'static, str>>) -> RunBefore<Self> {
        RunBefore(self, name.into())
    }

    fn run_after(self, name: impl Into<Cow<'static, str>>) -> RunAfter<Self> {
        RunAfter(self, name.into())
    }
}

pub struct RunBefore<T: Pass>(pub T, Cow<'static, str>);
pub struct RunAfter<T: Pass>(pub T, Cow<'static, str>);

impl<T: Pass> Pass for RunBefore<T> {
    fn encode(&self, res: &RenderResMap, encoder: &mut CommandEncoder, gpu: &Gpu) {
        self.0.encode(res, encoder, gpu);
    }

    fn node_builder(&self) -> (impl FnOnce(PassNode) -> PassNode + 'static) {
        let name = self.1.clone();
        let node_builder = self.0.node_builder();
        move |node| node_builder(node).run_before(name)
    }
}

impl<T: Pass> Pass for RunAfter<T> {
    fn encode(&self, res: &RenderResMap, encoder: &mut CommandEncoder, gpu: &Gpu) {
        self.0.encode(res, encoder, gpu);
    }

    fn node_builder(&self) -> (impl FnOnce(PassNode) -> PassNode + 'static) {
        let name = self.1.clone();
        let node_builder = self.0.node_builder();
        move |node| node_builder(node).run_after(name)
    }
}
