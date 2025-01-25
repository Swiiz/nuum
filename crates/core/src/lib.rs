pub mod platform;
pub use mint as maths;

pub trait Controller<T> {
    fn run(&mut self, input: T);
}
