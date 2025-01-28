use std::{
    any::{type_name, Any},
    borrow::BorrowMut,
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    usize, vec,
};

use nuum_gpu::wgpu::{SurfaceTexture, TextureView};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ResId(usize);

#[derive(Debug, Copy, Clone)]
pub struct ResHandle<T>(ResId, PhantomData<T>);

impl<T> ResHandle<T> {
    pub fn read(&self) -> ReadRes<T> {
        ReadRes(self.0, PhantomData)
    }

    pub fn write(&self) -> WriteRes<T> {
        WriteRes(self.0, PhantomData)
    }

    pub fn move_(&self) -> MoveRes<T> {
        MoveRes(self.0, PhantomData)
    }

    pub fn result(&self) -> ResultRes<T> {
        ResultRes(self.0, PhantomData)
    }
}

pub trait ResAccessor: Into<ResId> {
    type Value<'a>
    where
        Self: 'a;
    fn try_access<'a>(&self, res: &'a RenderResMap) -> Option<Self::Value<'a>>;
    fn from_id(id: usize) -> Self;
}

macro_rules! impl_res_handles {
    ($($name:ident)*) => {
        $(
          pub struct $name<T: 'static>(ResId, PhantomData<T>);

          impl<T> Copy for $name<T> {}
          impl<T> Clone for $name<T> {
              fn clone(&self) -> Self {
                  *self
              }
          }

          impl<T: 'static> $name<T> {
              #[allow(dead_code)]
              pub(crate) fn new(index: usize) -> Self {
                  Self(ResId(index), PhantomData)
              }
          }

          impl<T: 'static> Into<ResId> for $name<T> {
              fn into(self) -> ResId {
                  self.0
              }
          }


        )*
    };
}
impl_res_handles!(ReadRes WriteRes MoveRes ResultRes);

impl<T: 'static> ResAccessor for ReadRes<T> {
    type Value<'a> = Ref<'a, T>;

    fn try_access<'a>(&self, res: &'a RenderResMap) -> Option<Self::Value<'a>> {
        Ref::filter_map(res.alloc.elems[self.0 .0].borrow(), |b| {
            Some(b.as_ref()?.downcast_ref::<T>().unwrap())
        })
        .ok()
    }

    fn from_id(id: usize) -> Self {
        Self::new(id)
    }
}

impl<T: 'static> ResAccessor for WriteRes<T> {
    type Value<'a> = RefMut<'a, T>;

    fn try_access<'a>(&self, res: &'a RenderResMap) -> Option<Self::Value<'a>> {
        RefMut::filter_map(res.alloc.elems[self.0 .0].borrow_mut(), |b| {
            Some(b.as_mut()?.downcast_mut::<T>().unwrap())
        })
        .ok()
    }

    fn from_id(id: usize) -> Self {
        Self::new(id)
    }
}

impl<T: 'static> ResAccessor for MoveRes<T> {
    type Value<'a> = T;

    fn try_access<'a>(&'a self, res: &'a RenderResMap) -> Option<Self::Value<'a>> {
        Some(
            *res.alloc.elems[self.0 .0]
                .borrow_mut()
                .take()?
                .downcast()
                .unwrap(),
        )
    }

    fn from_id(id: usize) -> Self {
        Self::new(id)
    }
}

pub struct ResultResValue<'a, T>(RefMut<'a, Option<Box<dyn Any>>>, PhantomData<T>);

impl<T: 'static> ResultResValue<'_, T> {
    pub fn replace(&mut self, value: T) -> Option<T> {
        self.0
            .replace(Box::new(value))
            .map(|b| *b.downcast().unwrap())
    }

    pub fn set(&mut self, value: Option<T>) -> Option<T> {
        std::mem::replace(&mut *self.0, value.map(|v| Box::new(v) as Box<dyn Any>))
            .map(|b| *b.downcast().unwrap())
    }
}

impl<T: 'static> ResAccessor for ResultRes<T> {
    type Value<'a> = ResultResValue<'a, T>;

    fn try_access<'a>(&self, res: &'a RenderResMap) -> Option<Self::Value<'a>> {
        Some(ResultResValue(
            res.alloc.elems[self.0 .0].borrow_mut(),
            PhantomData,
        ))
    }

    fn from_id(id: usize) -> Self {
        Self::new(id)
    }
}

pub struct RenderGraphAlloc {
    elems: Vec<RefCell<Option<Box<dyn Any>>>>,
}

impl Default for RenderGraphAlloc {
    fn default() -> Self {
        Self {
            elems: vec![RefCell::new(None), RefCell::new(None)],
        }
    }
}

impl RenderGraphAlloc {
    pub fn frame_view(&self) -> ResHandle<TextureView> {
        ResHandle(ResId(0), PhantomData)
    }

    pub fn frame_surface_texture(&self) -> ResHandle<SurfaceTexture> {
        ResHandle(ResId(1), PhantomData)
    }

    pub fn push<T: 'static>(&mut self, value: Option<T>) -> ResHandle<T> {
        let index = self.elems.len();
        self.elems
            .push(RefCell::new(value.map(|v| Box::new(v) as Box<dyn Any>)));

        ResHandle(ResId(index), PhantomData)
    }
}

pub struct RenderResMap {
    alloc: RenderGraphAlloc,
}

impl RenderResMap {
    pub(super) fn not_ready(alloc: RenderGraphAlloc) -> Self {
        Self { alloc }
    }

    pub(super) fn prepare(
        &mut self,
        frame_view: TextureView,
        frame_surface_texture: SurfaceTexture,
    ) {
        self.alloc.elems[0].get_mut().replace(Box::new(frame_view));

        self.alloc.elems[1]
            .get_mut()
            .replace(Box::new(frame_surface_texture));
    }

    pub(super) fn finish(&mut self) -> (TextureView, SurfaceTexture) {
        let frame_view: TextureView = *self.alloc.elems[0]
            .get_mut()
            .take()
            .expect("Frame view cannot be consumed by render graph")
            .downcast()
            .unwrap();

        let frame_surface_texture: SurfaceTexture = *self.alloc.elems[1]
            .get_mut()
            .take()
            .expect("Frame surface texture cannot be consumed by render graph")
            .downcast()
            .unwrap();

        (frame_view, frame_surface_texture)
    }

    /// Access a resource in the render graph.
    ///
    /// # Panics
    ///
    /// if the resource is not present (uninit / moved) while using `ReadRes`, `WriteRes` or `MoveRes`. <br/>
    /// if the resources is currently in read while using `WriteRes` `MoveRes` or `ResultRes`. <br/>
    /// if the resource is currently in write while using `ReadRes`, `WriteRes`, `MoveRes` or `ResultRes`. <br/>
    ///
    ///  ## Advice
    ///
    /// Those safety rules are automatically met inside properly configured render graph nodes. <br/>
    /// Manual accessing for data feeding the render graph should be done with caution!
    ///
    pub fn access<'a, T: ResAccessor>(&'a self, res: &T) -> T::Value<'a> {
        res.try_access(self).unwrap_or_else(|| {
            panic!(
                "Tried to access uninit render graph resource {}",
                type_name::<T>()
            )
        })
    }

    pub fn try_access<'a, T: ResAccessor>(&'a self, res: &T) -> Option<T::Value<'a>> {
        res.try_access(self)
    }
}
