
use crate::RenderEvent;

/// Renderer with platform specific implementation, allowing access to platform event such as input
pub trait NativeRenderer<T, P, Inner> {
    fn on_platform_event(&mut self, input: &mut P);
    fn render_port(&mut self, event: &mut RenderEvent<T>, inner: &mut Inner);
}

macro_rules! impl_for_tuples {
    ($($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<
            T,
            P,
            Inner,
            $( $t ),*
        > NativeRenderer<T, P, Inner> for ( $( $t, )* )
        where
            $( $t: NativeRenderer<T, P, Inner> ),*
        {
            fn on_platform_event(&mut self, _input: &mut P) {
                let ($($t,)*) = self;
                $( $t.on_platform_event(_input); )*
            }
            fn render_port(&mut self, _event: &mut RenderEvent<T>, _inner: &mut Inner) {
                 let ($($t,)*) = self;
                $( $t.render_port(_event, _inner); )*
            }
        }
    };
}

impl_for_tuples!();
impl_for_tuples!(A);
impl_for_tuples!(A, B);
impl_for_tuples!(A, B, C);
impl_for_tuples!(A, B, C, D);
impl_for_tuples!(A, B, C, D, E);
impl_for_tuples!(A, B, C, D, E, F);
impl_for_tuples!(A, B, C, D, E, F, G);
impl_for_tuples!(A, B, C, D, E, F, G, H);
