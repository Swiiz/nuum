pub mod event;
pub mod platform;
pub use mint as maths;

pub trait Controller<Event> {
    fn run(&mut self, input: Event);
}

pub trait Port<'a, Event, Inner> {
    fn port(&mut self, event: &'a mut Event, inner: &mut Inner);
}

pub struct Adapter<Port, Inner> {
    pub ports: Port,
    pub inner: Inner,
}

impl<P, I, E> Controller<E> for Adapter<P, I>
where
    P: for<'a> Port<'a, E, I>,
{
    fn run(&mut self, mut input: E) {
        self.ports.port(&mut input, &mut self.inner);
    }
}

impl<Event, F: FnMut(Event)> Controller<Event> for F {
    fn run(&mut self, input: Event) {
        self(input);
    }
}

impl<'a, Event, Inner, F: FnMut(&mut Event, &mut Inner)> Port<'a, Event, Inner> for F {
    fn port(&mut self, event: &mut Event, inner: &mut Inner) {
        self(event, inner);
    }
}

impl<Event> Controller<Event> for () {
    fn run(&mut self, _input: Event) {}
}

macro_rules! impl_port_tuples {
    ($($t:ident $l:lifetime),*) => {
        impl<'_a, _E, _I, $( $t: for<'a> Port<'a, _E, _I> ),*> Port<'_a, _E, _I> for ( $( $t, )* )
        where  {
            fn port(& mut self, mut _event: & mut _E, _inner: & mut _I) {
                #[allow(non_snake_case)]
                let ($($t,)*) = self;
                $(
                    $t.port(_event, _inner);
                )*
            }
        }
    };
}

impl_port_tuples!(A 'a);
impl_port_tuples!(A 'a, B 'b);
impl_port_tuples!(A 'a, B 'b, C 'c);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd, E 'e);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd, E 'e, F 'f);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd, E 'e, F 'f, G 'g);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd, E 'e, F 'f, G 'g, H 'h);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd, E 'e, F 'f, G 'g, H 'h, I 'i);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd, E 'e, F 'f, G 'g, H 'h, I 'i, J 'j);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd, E 'e, F 'f, G 'g, H 'h, I 'i, J 'j, K 'k);
impl_port_tuples!(A 'a, B 'b, C 'c, D 'd, E 'e, F 'f, G 'g, H 'h, I 'i, J 'j, K 'k, L 'l);
