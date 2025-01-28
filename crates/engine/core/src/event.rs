#[macro_export]
macro_rules! impl_event_kind {
    ($($name:ident)*) => {
        $(
          pub struct $name<T> {
              pub inner: T,
              pub dt: std::time::Duration,
          }

          impl<T> std::ops::Deref for $name<T> {
              type Target = T;

              fn deref(&self) -> &Self::Target {
                  &self.inner
              }
          }

          impl<T> std::ops::DerefMut for $name<T> {
              fn deref_mut(&mut self) -> &mut Self::Target {
                  &mut self.inner
              }
          }
        )*
    };
}

impl_event_kind!(Render Update);

#[macro_export]
macro_rules! impl_schema {
    ($name:ident := $first:ident $(+ $kind:ident)*) => {
        #[allow(non_snake_case)]
        pub struct $name<$first, $($kind,)* Inner> {
            $first: $first,
            $($kind: $kind,)*
            inner: Inner
        }

        #[allow(non_snake_case)]
        impl<$first, $($kind,)* Inner> $name<$first, $($kind,)* Inner> {
            pub fn new(inner: Inner, $first: $first, $($kind: $kind,)*) -> Self {
                Self {
                    inner, $first, $($kind,)*
                }
            }
        }

        $crate::impl_schema!(@foreach $name $first $($kind)* : ($first $($kind)*));
    };
    (@foreach $name:ident $($spe:ident)* : $kinds:tt) => {
        $(
            $crate::impl_schema!(@solo $name $spe $kinds);
        )*
    };
    (@solo $name:ident $spe:ident ($($kind:ident)*)) => {
        impl<$($kind,)* Inner, Event> $crate::Controller<self::$spe<Event>> for $name<$($kind,)* Inner>
        where
            $spe: for<'a> $crate::Port<'a, self::$spe<Event>, Inner>,
        {
            fn run(&mut self, mut event: self::$spe<Event>) {
                self.$spe.port(&mut event, &mut self.inner);
            }
        }
    }
}
