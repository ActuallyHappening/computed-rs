use std::sync::OnceLock;

pub use static_assertions;

pub use computed_macros::*;

pub trait ComputableContainer {
    type Output;
    fn invalidate(&mut self);
    fn get_or_init(&mut self, f: impl FnOnce() -> Self::Output) -> &Self::Output;
}

impl<T> ComputableContainer for OnceLock<T> {
    type Output = T;
    fn invalidate(&mut self) {
        self.take();
    }
    fn get_or_init(&mut self, f: impl FnOnce() -> Self::Output) -> &Self::Output {
        OnceLock::get_or_init(self, f)
    }
}

// impl for Option<T>, OnceCell<T>, Cell<T>
impl<T> ComputableContainer for Option<T> {
    type Output = T;
    fn invalidate(&mut self) {
        *self = None;
    }
    fn get_or_init(&mut self, f: impl FnOnce() -> Self::Output) -> &Self::Output {
        self.get_or_insert_with(f)
    }
}

impl<T> ComputableContainer for std::cell::OnceCell<T> {
		type Output = T;
    fn invalidate(&mut self) {
        self.take();
    }
    fn get_or_init(&mut self, f: impl FnOnce() -> Self::Output) -> &Self::Output {
        std::cell::OnceCell::get_or_init(self, f)
    }
}
